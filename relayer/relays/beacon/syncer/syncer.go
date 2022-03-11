package syncer

import (
	"io"
	"net/http"

	"github.com/sirupsen/logrus"
)

type Syncer interface {
	GetHeader() error
}

type Sync struct {
	httpClient http.Client
	endpoint   string
}

func New(endpoint string) Sync {
	return Sync{
		http.Client{},
		endpoint,
	}
}

func (s *Sync) GetHeader() error {
	client := &http.Client{}
	req, _ := http.NewRequest(http.MethodGet, s.endpoint+"/eth/v1/beacon/headers/head", nil)
	req.Header.Set("accept", "application/json")
	res, err := client.Do(req)
	if err != nil {
		logrus.WithError(err).Error("failed to do http request")

		return nil
	}

	if res.StatusCode != http.StatusOK {
		logrus.Error("request to beacon node failed")

		return nil
	}

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		logrus.Error("unable to get response body")

		return nil
	}
	bodyString := string(bodyBytes)

	logrus.WithField("body", bodyString).Info("request to get beacon header")
}
