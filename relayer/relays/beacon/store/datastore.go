package store

import (
	"database/sql"
	"errors"
	"fmt"
	"os"
	"time"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/protocol"

	_ "github.com/mattn/go-sqlite3"
)

const BeaconStateDir = "states"
const BeaconStateFilename = "beacon_state_%d.ssz"
const BeaconStoreName = "beacon-state"

type BeaconStore interface {
	Connect() error
	Close()
	FindBeaconStateWithinSyncPeriod(slot, boundary uint64, findMin bool) (StoredBeaconData, error)
	GetBeaconStateData(slot uint64) ([]byte, error)
	WriteEntry(attestedSlot, finalizedSlot uint64, attestedStateData, finalizedStateData []byte) error
}

type BeaconState struct {
	ID                     uint64
	AttestedSlot           uint64
	FinalizedSlot          uint64
	AttestedSyncPeriod     uint64
	FinalizedSyncPeriod    uint64
	AttestedStateFilename  string
	FinalizedStateFilename string
	Timestamp              time.Time
}

type StoredBeaconData struct {
	AttestedSlot         uint64
	FinalizedSlot        uint64
	AttestedBeaconState  []byte
	FinalizedBeaconState []byte
}

type Store struct {
	location   string
	maxEntries uint64
	db         *sql.DB
	protocol   protocol.Protocol
}

func New(location string, maxEntries uint64, protocol protocol.Protocol) Store {
	return Store{
		location,
		maxEntries,
		nil,
		protocol,
	}
}

func (s *Store) Connect() error {
	var err error
	s.db, err = sql.Open("sqlite3", s.location+BeaconStoreName)
	if err != nil {
		return err
	}

	err = s.createTable()
	if err != nil {
		return err
	}

	err = createBeaconStateDir(s.location + BeaconStateDir)
	if err != nil {
		return err
	}

	return nil
}

func (s *Store) Close() {
	_ = s.db.Close()
}

// FindBeaconStateWithinSyncPeriod finds an attested and a finalized header pair within the same sync period.
// bool findMin specifies whether the largest or smallest slot should be found. if findMin = true, the earliest
// block in the sync committee will be returned, otherwise the largest. This is used for FinalizedUpdates, where
// the latest block ideally wants to be synced, and for SyncCommitteeUpdates, where the earliest slot with the
// next sync committee wants to be located.
func (s *Store) FindBeaconStateWithinSyncPeriod(slot, boundary uint64, findMin bool) (StoredBeaconData, error) {
	var data StoredBeaconData

	var query string
	if findMin {
		query = `SELECT MIN(attested_slot), attested_slot, finalized_slot, attested_state_filename, finalized_state_filename FROM beacon_state WHERE attested_slot >= ? AND attested_slot < ?`
	} else {
		query = `SELECT MAX(attested_slot), attested_slot, finalized_slot, attested_state_filename, finalized_state_filename FROM beacon_state WHERE attested_slot >= ? AND attested_slot < ?`
	}
	var min uint64
	var attestedSlot uint64
	var finalizedSlot uint64
	var attestedStateFilename string
	var finalizedStateFilename string
	err := s.db.QueryRow(query, slot, boundary).Scan(&min, &attestedSlot, &finalizedSlot, &attestedStateFilename, &finalizedStateFilename)
	if err != nil {
		return data, fmt.Errorf("no match found")
	}

	attestedState, err := s.ReadStateFile(attestedStateFilename)
	if err != nil {
		return data, fmt.Errorf("could not read beacon file %s", attestedStateFilename)
	}

	finalizedState, err := s.ReadStateFile(finalizedStateFilename)
	if err != nil {
		return data, fmt.Errorf("could not read beacon file %s", finalizedStateFilename)
	}

	data = StoredBeaconData{
		AttestedSlot:         attestedSlot,
		FinalizedSlot:        finalizedSlot,
		AttestedBeaconState:  attestedState,
		FinalizedBeaconState: finalizedState,
	}

	return data, nil
}

// GetBeaconStateData finds a beacon state at a slot.
func (s *Store) GetBeaconStateData(slot uint64) ([]byte, error) {
	query := `SELECT attested_slot, finalized_slot, attested_state_filename, finalized_state_filename FROM beacon_state WHERE attested_slot = ? OR finalized_slot = ? LIMIT 1`
	var attestedSlot uint64
	var finalizedSlot uint64
	var attestedStateFilename string
	var finalizedStateFilename string
	err := s.db.QueryRow(query, slot, slot).Scan(&attestedSlot, &finalizedSlot, &attestedStateFilename, &finalizedStateFilename)
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			// No finalized slots found within the range
			return nil, fmt.Errorf("no match found")
		}
		return nil, err
	}

	if attestedSlot == slot {
		return s.ReadStateFile(attestedStateFilename)
	}

	if finalizedSlot == slot {
		return s.ReadStateFile(finalizedStateFilename)
	}

	return nil, fmt.Errorf("no beacon state found")
}

func (s *Store) WriteEntry(attestedSlot, finalizedSlot uint64, attestedStateData, finalizedStateData []byte) error {
	err := s.writeStateFile(attestedSlot, attestedStateData)
	if err != nil {
		return err
	}
	err = s.writeStateFile(finalizedSlot, finalizedStateData)
	if err != nil {
		return err
	}

	attestedSyncPeriod := s.protocol.ComputeSyncPeriodAtSlot(attestedSlot)
	finalizedSyncPeriod := s.protocol.ComputeSyncPeriodAtSlot(finalizedSlot)

	return s.storeUpdate(attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod)
}

func (s *Store) DeleteStateFile(filename string) error {
	err := os.Remove(s.location + BeaconStateDir + "/" + filename)
	if err != nil {
		return fmt.Errorf("remove file: %w", err)
	}

	return nil
}

func (s *Store) ReadStateFile(filename string) ([]byte, error) {
	data, err := os.ReadFile(s.location + BeaconStateDir + "/" + filename)
	if err != nil {
		return nil, fmt.Errorf("read file: %w", err)
	}

	return data, nil
}

func (s *Store) PruneOldStates() ([]uint64, error) {
	selectSQL := fmt.Sprintf(`
	SELECT id, attested_slot, finalized_slot, attested_sync_period, finalized_sync_period, attested_state_filename, finalized_state_filename
	FROM beacon_state
	WHERE id NOT IN (
		SELECT id FROM beacon_state
		ORDER BY attested_slot DESC
		LIMIT %d
	)`, s.maxEntries)

	rows, err := s.db.Query(selectSQL)
	if err != nil {
		return nil, fmt.Errorf("failed to select oldest entries: %w", err)
	}
	defer rows.Close()

	var deleteSlots []uint64
	for rows.Next() {
		var entry BeaconState
		if err := rows.Scan(&entry.ID, &entry.AttestedSlot, &entry.FinalizedSlot, &entry.AttestedSyncPeriod, &entry.FinalizedSyncPeriod, &entry.AttestedStateFilename, &entry.FinalizedStateFilename); err != nil {
			return nil, fmt.Errorf("failed to scan row: %w", err)
		}
		deleteSlots = append(deleteSlots, entry.AttestedSlot)
		deleteSlots = append(deleteSlots, entry.FinalizedSlot)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating rows: %w", err)
	}

	for _, slot := range deleteSlots {
		err := s.DeleteStateFile(fmt.Sprintf(BeaconStateFilename, slot))
		if err != nil {
			return nil, err
		}
	}

	// Then, delete those rows
	pruneSQL := fmt.Sprintf(`
		DELETE FROM beacon_state
		WHERE id IN (
			SELECT id FROM beacon_state
			WHERE id NOT IN (
				SELECT id FROM beacon_state
				ORDER BY timestamp DESC
				LIMIT %d
			)
		)`, s.maxEntries)
	_, err = s.db.Exec(pruneSQL)
	if err != nil {
		return nil, fmt.Errorf("failed to prune oldest entries: %w", err)
	}

	return deleteSlots, nil
}

func createBeaconStateDir(dirPath string) error {
	if _, err := os.Stat(dirPath); os.IsNotExist(err) {
		return os.MkdirAll(dirPath, os.ModePerm)
	}
	return nil
}

func (s *Store) createTable() error {
	sqlStmt := `CREATE TABLE IF NOT EXISTS beacon_state (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		attested_slot INTEGER NOT NULL,
		finalized_slot INTEGER NOT NULL,
		attested_sync_period INTEGER NOT NULL,
		finalized_sync_period INTEGER NOT NULL,
		attested_state_filename TEXT NOT NULL,
		finalized_state_filename TEXT NOT NULL,
		timestamp INTEGER DEFAULT (strftime('%s', 'now'))
	);`
	_, err := s.db.Exec(sqlStmt)
	if err != nil {
		return err
	}

	return nil
}

func (s *Store) writeStateFile(slot uint64, data []byte) error {
	err := os.WriteFile(s.location+BeaconStateDir+"/"+fmt.Sprintf(BeaconStateFilename, slot), data, 0644)
	if err != nil {
		return fmt.Errorf("write to file: %w", err)
	}

	return nil
}

func (s *Store) storeUpdate(attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod uint64) error {
	attestedStateFileName := fmt.Sprintf(BeaconStateFilename, attestedSlot)
	finalizedStateFileName := fmt.Sprintf(BeaconStateFilename, finalizedSlot)

	insertStmt := `INSERT INTO beacon_state (attested_slot, finalized_slot,  attested_sync_period, finalized_sync_period, attested_state_filename, finalized_state_filename) VALUES (?, ?, ?, ?, ?, ?)`
	stmt, err := s.db.Prepare(insertStmt)
	if err != nil {
		return err
	}
	defer stmt.Close()

	_, err = stmt.Exec(attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod, attestedStateFileName, finalizedStateFileName)
	if err != nil {
		return err
	}

	return nil
}
