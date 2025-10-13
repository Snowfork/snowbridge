package beefy

import (
	"sort"
	"sync"
	"time"

	"golang.org/x/sync/semaphore"
)

type TaskState uint

const (
	Pending TaskState = iota
	InProgress
	Completed
	Failed
	Canceled
)

type TaskInfo struct {
	nonce     uint64 // Unique identifier for the task
	req       *Request
	status    TaskState
	timestamp uint64 // Unix timestamp of when the task was created
}

type TaskMap struct {
	mu            sync.RWMutex
	data          map[uint64]*TaskInfo
	limit         uint64 // Maximum number of tasks allowed
	mergePeriod   uint64 // The merge period during which a previous task can be merged
	expiredPeriod uint64 // The expiration period after which merging is not allowed
	sem           *semaphore.Weighted
	lastUpdated   uint64 // Last updated timestamp of a successful task
}

func NewTaskMap(limit, mergePeriod, expiredPeriod uint64) *TaskMap {
	return &TaskMap{
		data:          make(map[uint64]*TaskInfo),
		limit:         limit,
		sem:           semaphore.NewWeighted(int64(limit)),
		mergePeriod:   mergePeriod,
		expiredPeriod: expiredPeriod,
	}
}

func (tm *TaskMap) Store(key uint64, req *Request) bool {
	tm.mu.Lock()
	defer tm.mu.Unlock()
	if len(tm.data) >= int(tm.limit) {
		return false
	}
	tm.data[key] = &TaskInfo{
		nonce:     key,
		req:       req,
		status:    Pending,
		timestamp: uint64(time.Now().Unix()),
	}
	return true
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

// Pop the next available task, clean up any completed or canceled tasks, and return nil if none are available.
// Merge previous tasks and mark them as skippable if any of the conditions are met:
// a. Outdated
// b. Just updated
// c. Can be replaced by a newer one while still unexpired
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
	findNextTask := func(current *TaskInfo) *TaskInfo {
		for index, key := range keys {
			if index < len(keys)-1 {
				task := tm.data[uint64(key)]
				if task.nonce == current.nonce {
					return tm.data[uint64(keys[index+1])]
				}
			}
		}
		return nil
	}
	outdated := func(task *TaskInfo) bool {
		return task.timestamp < tm.lastUpdated
	}
	justUpdated := func(task *TaskInfo) bool {
		nextTask := findNextTask(task)
		if nextTask != nil {
			return tm.lastUpdated > 0 && task.timestamp > tm.lastUpdated && task.timestamp-tm.lastUpdated < tm.mergePeriod
		}
		return false
	}
	canBeReplaced := func(task *TaskInfo) bool {
		nextTask := findNextTask(task)
		if nextTask != nil {
			return nextTask.timestamp > task.timestamp && nextTask.timestamp-task.timestamp < tm.mergePeriod
		}
		return false
	}
	unexpired := func(task *TaskInfo) bool {
		return tm.lastUpdated > 0 && task.timestamp > tm.lastUpdated && task.timestamp-tm.lastUpdated < tm.expiredPeriod
	}
	for _, key := range keys {
		task, _ := tm.data[uint64(key)]
		if outdated(task) || justUpdated(task) || (canBeReplaced(task) && unexpired(task)) {
			task.req.Skippable = true
		}
	}

	for _, k := range keys {
		task := tm.data[uint64(k)]
		if task.status == Completed || task.status == Canceled {
			delete(tm.data, uint64(k))
		}
		if task.status == Pending || task.status == Failed {
			task.status = InProgress
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

func (tm *TaskMap) SetLastUpdated(key uint64) {
	tm.mu.RLock()
	defer tm.mu.RUnlock()
	val, ok := tm.data[key]
	if ok && val.timestamp > tm.lastUpdated {
		tm.lastUpdated = val.timestamp
	}
}
