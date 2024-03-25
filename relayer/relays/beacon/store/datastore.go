package store

import (
	"database/sql"
	"fmt"
	"os"
	"strings"
	"time"
)

const BeaconStateDir = "states"
const BeaconStateFilename = "beacon_state_%d.ssz"

type BeaconStore interface {
	Connect() error
	Close()
	StoreUpdate(attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod uint64) error
	FindBeaconStateWithinSyncPeriodRange(baseSlot, slotRange uint64, findMax bool) (StoredBeaconData, error)
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
}

func New(location string, maxEntries uint64) Store {
	return Store{
		location,
		maxEntries,
		nil,
	}
}

func (s *Store) Connect() error {
	var err error
	s.db, err = sql.Open("sqlite3", s.location+"beacon-state")
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

func (s *Store) StoreUpdate(attestedSlot, finalizedSlot, attestedSyncPeriod, finalizedSyncPeriod uint64) error {
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

// Find the latest finalized header within the same sync committee.
func (s *Store) FindBeaconStateWithinSyncPeriodRange(baseSlot, boundarySlot uint64, findMax bool) (StoredBeaconData, error) {
	var data StoredBeaconData

	var query string
	if findMax {
		query = `SELECT MAX(attested_slot), finalized_slot, attested_state_filename, finalized_state_filename FROM beacon_state WHERE attested_slot >= ? AND attested_slot <= ?`
	} else {
		query = `SELECT MIN(attested_slot), finalized_slot, attested_state_filename, finalized_state_filename FROM beacon_state WHERE attested_slot >= ? AND attested_slot <= ?`
	}
	var attestedSlot uint64
	var finalizedSlot uint64
	var attestedStateFilename string
	var finalizedStateFilename string
	err := s.db.QueryRow(query, baseSlot, boundarySlot).Scan(&attestedSlot, &finalizedSlot, &attestedStateFilename, &finalizedStateFilename)
	if err != nil {
		if err == sql.ErrNoRows {
			// No finalized slots found within the range
			return data, fmt.Errorf("no match found")
		}
		return data, err
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

func (s *Store) WriteStateFile(slot uint64, data []byte) error {
	err := os.WriteFile(s.location+BeaconStateDir+"/"+fmt.Sprintf(BeaconStateFilename, slot), data, 0644)
	if err != nil {
		return fmt.Errorf("write to file: %w", err)
	}

	return nil
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
	SELECT id, attested_slot, finalized_slot, attested_sync_period, finalized_sync_period, attested_state_filename, finalized_state_filename, timestamp
	FROM beacon_state
	WHERE id NOT IN (
		SELECT id FROM beacon_state
		ORDER BY timestamp DESC
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
		var timestampInt64 int64
		if err := rows.Scan(&entry.ID, &entry.AttestedSlot, &entry.FinalizedSlot, &entry.AttestedSyncPeriod, &entry.FinalizedSyncPeriod, &entry.AttestedStateFilename, &entry.FinalizedStateFilename, &timestampInt64); err != nil {
			return nil, fmt.Errorf("failed to scan row: %w", err)
		}
		deleteSlots = append(deleteSlots, entry.AttestedSlot)
		deleteSlots = append(deleteSlots, entry.FinalizedSlot)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating rows: %w", err)
	}

	var slotsForQuery []string
	for _, slot := range deleteSlots {
		slotsForQuery = append(slotsForQuery, fmt.Sprintf("%d", slot))
	}
	slotsStr := "(" + strings.Join(slotsForQuery, ",") + ")"
	// Query to find any matching AttestedSlot or FinalizedSlot
	query := fmt.Sprintf(`SELECT DISTINCT attested_slot FROM beacon_state WHERE attested_slot IN %s
	UNION
	SELECT DISTINCT finalized_slot FROM beacon_state WHERE finalized_slot IN %s`, slotsStr, slotsStr)

	existingRows, err := s.db.Query(query)
	if err != nil {
		return nil, err
	}
	defer existingRows.Close()

	// Create a map of found slots to efficiently check for existence
	foundSlots := make(map[uint64]bool)
	for existingRows.Next() {
		var slot uint64
		if err := existingRows.Scan(&slot); err != nil {
			return nil, err
		}
		foundSlots[slot] = true
	}

	for _, slot := range deleteSlots {
		if !foundSlots[slot] {
			err := s.DeleteStateFile(fmt.Sprintf(BeaconStateFilename, slot))
			if err != nil {
				return nil, err
			}
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
