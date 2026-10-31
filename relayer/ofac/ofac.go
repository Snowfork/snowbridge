package ofac

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	log "github.com/sirupsen/logrus"
)

const (
	defaultEndpoint = "https://public.chainalysis.com/api/v1/address"
	requestTimeout  = 10 * time.Second
)

type OFAC struct {
	enabled bool
	apiKey  string
	client  *http.Client
	endpoint string
}

type Response struct {
	Identifications []struct {
		Category    string `json:"category"`
		Name        string `json:"name"`
		Description string `json:"description"`
		URL         string `json:"url"`
	} `json:"identifications"`
}

func New(enabled bool, apiKey string) *OFAC {
	return &OFAC{
		enabled:  enabled,
		apiKey:   apiKey,
		client:   &http.Client{Timeout: requestTimeout},
		endpoint: defaultEndpoint,
	}
}

func (o OFAC) IsBanned(source string, destinations []string) (bool, error) {
	if !o.enabled {
		return false, nil
	}

	if source != "" {
		isSourcedBanned, err := o.isOFACListed(source)
		if err != nil {
			return true, err
		}
		if isSourcedBanned {
			log.WithField("source", source).Warn("found ofac banned source address")
			return true, nil
		}
	}

	for _, destination := range destinations {
		if destination != "" {
			isDestinationBanned, err := o.isOFACListed(destination)
			if err != nil {
				return true, err
			}
			if isDestinationBanned {
				log.WithField("destination", destination).Warn("found ofac banned destination address")
				return true, nil
			}
		}
	}

	return false, nil
}

func (o OFAC) isOFACListed(address string) (bool, error) {
	endpoint := fmt.Sprintf("%s/%s", strings.TrimRight(o.endpoint, "/"), url.PathEscape(address))
	req, err := http.NewRequest("GET", endpoint, nil)
	if err != nil {
		return true, err
	}

	req.Header.Add("Accept", "application/json")
	req.Header.Add("X-API-Key", o.apiKey)

	resp, err := o.client.Do(req)
	if err != nil {
		return true, err
	}
	defer resp.Body.Close()
	if resp.StatusCode < http.StatusOK || resp.StatusCode >= http.StatusMultipleChoices {
		return true, fmt.Errorf("OFAC API returned unexpected status %s", resp.Status)
	}

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
