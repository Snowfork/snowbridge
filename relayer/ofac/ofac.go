package ofac

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

type OFAC struct {
	enabled bool
	apiKey  string
}

type Response struct {
	Identifications []struct {
		Category    string `json:"category"`
		Name        string `json:"name"`
		Description string `json:"description"`
		URL         string `json:"url"`
	} `json:"identifications"`
}

func New(enabled bool, apiKey string) OFAC {
	return OFAC{enabled, apiKey}
}

func (o OFAC) IsBanned(source, destination string) (bool, error) {
	if !o.enabled {
		return false, nil
	}

	isSourcedBanned, err := o.checkOFAC(source)
	if err != nil {
		return true, err
	}
	if isSourcedBanned {
		return true, nil
	}

	isDestinationBanned, err := o.checkOFAC(destination)
	if err != nil {
		return true, err
	}
	if isDestinationBanned {
		return true, nil
	}

	return false, nil
}

func (o OFAC) checkOFAC(address string) (bool, error) {
	client := &http.Client{}

	req, err := http.NewRequest("GET", fmt.Sprintf("https://public.chainalysis.com/api/v1/address/%s", address), nil)
	if err != nil {
		return true, err
	}

	req.Header.Add("Accept", "application/json")
	req.Header.Add("X-API-Key", o.apiKey)

	resp, err := client.Do(req)
	if err != nil {
		return true, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return true, err
	}

	var response Response
	err = json.Unmarshal(body, &response)
	if err != nil {
		return true, err
	}

	return len(response.Identifications) > 0, nil
}
