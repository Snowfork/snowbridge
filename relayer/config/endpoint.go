package config

import "strings"

// ParseEndpoints splits a comma-separated endpoint string into a slice of endpoints.
// Leading/trailing whitespace is trimmed from each endpoint.
func ParseEndpoints(endpoint string) []string {
	parts := strings.Split(endpoint, ",")
	var endpoints []string
	for _, p := range parts {
		p = strings.TrimSpace(p)
		if p != "" {
			endpoints = append(endpoints, p)
		}
	}
	return endpoints
}
