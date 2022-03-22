package syncer

/**
{"data":{"validators":["145296","252393","190192","138469","172853","..."]]}
**/

type SyncCommitteeResponse struct {
	Data struct {
		Validators []string `json:"validators"`
	} `json:"data"`
}

type SyncCommittee struct {
	Indexes []uint64
}