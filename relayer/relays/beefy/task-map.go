package beefy

import (
	"sort"
	"sync"
	"time"

	"golang.org/x/sync/semaphore"
)

type TaskState uint

const (
	TaskPending TaskState = iota
	TaskInProgress
	TaskCompleted
	TaskFailed
	TaskCanceled
)

type TaskInfo struct {
	nonce     uint64 // Unique identifier for the task
	req       *Request
	status    TaskState
	timestamp uint64 // Unix timestamp of when the task was created
}

type TaskMap struct {
	mu          sync.RWMutex
	data        map[uint64]*TaskInfo
	limit       uint64 // Maximum number of tasks allowed
	mergePeriod uint64 // Merge tasks based upon the interval if the new task is close to the previous one
	sem         *semaphore.Weighted
	lastUpdated uint64 // Last updated timestamp of a successful task
}

func NewTaskMap(limit uint64, mergePeriod uint64) *TaskMap {
	return &TaskMap{
		data:        make(map[uint64]*TaskInfo),
		limit:       limit,
		sem:         semaphore.NewWeighted(int64(limit)),
		mergePeriod: mergePeriod,
	}
}

func (tm *TaskMap) Store(key uint64, task *Request) {
	tm.mu.Lock()
	defer tm.mu.Unlock()
	if len(tm.data) >= int(tm.limit) {
		return
	}
	tm.data[key] = &TaskInfo{
		nonce:     key,
		req:       task,
		status:    TaskPending,
		timestamp: uint64(time.Now().Unix()),
	}
}

func (tm *TaskMap) Load(key uint64) (*TaskInfo, bool) {
	tm.mu.RLock()
	defer tm.mu.RUnlock()
	val, ok := tm.data[key]
	return val, ok
}

func (tm *TaskMap) Delete(key uint64) {
	tm.mu.Lock()
	defer tm.mu.Unlock()
	delete(tm.data, key)
}

func (tm *TaskMap) Full() bool {
	tm.mu.Lock()
	defer tm.mu.Unlock()
	if len(tm.data) >= int(tm.limit) {
		return true
	}
	return false
}

// Pop the next available task, or return nil if none are available.
func (tm *TaskMap) Pop() *TaskInfo {
	tm.mu.RLock()
	defer tm.mu.RUnlock()
	if len(tm.data) == 0 {
		return nil
	}
	keys := make([]int, 0, len(tm.data))
	for k := range tm.data {
		keys = append(keys, int(k))
	}
	sort.Ints(keys)
	for _, k := range keys {
		task := tm.data[uint64(k)]
		if task.status == TaskCompleted || task.status == TaskCanceled {
			delete(tm.data, uint64(k))
		}
		if task.status == TaskPending || task.status == TaskFailed {
			task.status = TaskInProgress
			return task
		}
	}
	return nil
}

func (tm *TaskMap) SetStatus(key uint64, status TaskState) {
	tm.mu.RLock()
	defer tm.mu.RUnlock()
	val, ok := tm.data[key]
	if ok {
		val.status = status
	}
}

func (tm *TaskMap) InspectAll() []*TaskInfo {
	tm.mu.RLock()
	defer tm.mu.RUnlock()
	tasks := make([]*TaskInfo, 0, len(tm.data))
	keys := make([]int, 0, len(tm.data))
	for k := range tm.data {
		keys = append(keys, int(k))
	}
	sort.Ints(keys)
	for _, k := range keys {
		task := tm.data[uint64(k)]
		tasks = append(tasks, task)
	}
	return tasks
}

// Merge previous tasks and mark them as skippable if the new task occurs close in time.
func (tm *TaskMap) Merge(key uint64) {
	tm.mu.Lock()
	defer tm.mu.Unlock()
	val, ok := tm.data[key]
	if !ok {
		return
	}
	if len(tm.data) < 2 {
		return
	}
	closeToPrevious := func(current, prev *TaskInfo) bool {
		return current.timestamp > prev.timestamp && current.timestamp-prev.timestamp < tm.mergePeriod
	}
	closeToLastUpdated := func(task *TaskInfo) bool {
		return tm.lastUpdated > 0 && task.timestamp > tm.lastUpdated && task.timestamp-tm.lastUpdated < 8*tm.mergePeriod
	}
	outdated := func(task *TaskInfo) bool {
		return task.timestamp < tm.lastUpdated
	}
	for k := range tm.data {
		if k != key {
			toSkip, _ := tm.data[k]
			if outdated(toSkip) || (closeToPrevious(val, toSkip) && closeToLastUpdated(toSkip)) {
				toSkip.req.Skippable = true
			}
		}
	}
}

func (tm *TaskMap) SetLastUpdated(key uint64) {
	tm.mu.RLock()
	defer tm.mu.RUnlock()
	val, ok := tm.data[key]
	if ok && val.timestamp > tm.lastUpdated {
		tm.lastUpdated = val.timestamp
	}
}
