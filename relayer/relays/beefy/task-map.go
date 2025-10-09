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
)

type TaskInfo struct {
	nonce     uint64 // Unique identifier for the task
	req       *Request
	status    TaskState
	timestamp uint64 // Unix timestamp of when the task was created
}

type TaskMap struct {
	mu    sync.RWMutex
	data  map[uint64]*TaskInfo
	limit int64 // Maximum number of tasks allowed
	sem   *semaphore.Weighted
}

func NewTaskMap(limit int64) *TaskMap {
	return &TaskMap{
		data:  make(map[uint64]*TaskInfo),
		limit: limit,
		sem:   semaphore.NewWeighted(limit),
	}
}

func (tm *TaskMap) Store(key uint64, task *Request) {
	tm.mu.Lock()
	defer tm.mu.Unlock()
	if int64(len(tm.data)) >= tm.limit {
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
	if int64(len(tm.data)) >= tm.limit {
		return true
	}
	return false
}

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
