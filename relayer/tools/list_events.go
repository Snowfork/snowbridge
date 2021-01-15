package main

import (
	"fmt"
	"os"

	gsrpc "github.com/snowfork/go-substrate-rpc-client/v2"

	"github.com/snowfork/go-substrate-rpc-client/v2/types"
)

type Events []Event

func (e Events) Len() int {
	return len(e)
}

func (e Events) Less(i, j int) bool {
	if e[i].ID[0] < e[j].ID[0] {
		return true
	}
	if e[i].ID[0] == e[j].ID[0] {
		if e[i].ID[1] < e[j].ID[1] {
			return true
		}
	}
	return false
}

func (e Events) Swap(i, j int) {
	tmp := e[i]
	e[i] = e[j]
	e[j] = tmp
}

func Map(vs []types.Type, f func(types.Type) string) []string {
	vsm := make([]string, len(vs))
	for i, v := range vs {
		vsm[i] = f(v)
	}
	return vsm
}

type Event struct {
	ID         [2]byte
	ModuleName string
	Name       string
	Fields     []string
}

func listEvents(m *types.Metadata) []Event {
	events := []Event{}
	for i, mod := range m.AsMetadataV11.Modules {
		if !mod.HasEvents {
			continue
		}
		for j, ev := range mod.Events {
			event := Event{
				ID:         [2]uint8{uint8(i), uint8(j)},
				ModuleName: string(mod.Name),
				Name:       string(ev.Name),
				Fields:     Map(ev.Args, func(v types.Type) string { return string(v) }),
			}
			events = append(events, event)
		}
	}
	return events
}

func main() {

	args := os.Args[1:]
	endpoint := args[0]

	api, err := gsrpc.NewSubstrateAPI(endpoint)
	if err != nil {
		panic(err)
	}

	m, err := api.RPC.State.GetMetadataLatest()
	if err != nil {
		panic(err)
	}

	if !m.IsMetadataV11 {
		panic("Unsupported metadata version")
	}

	events := listEvents(m)

	for i, ev := range events {
		fmt.Printf("%s %s\n", ev.ModuleName, ev.Name)
		fmt.Printf("%v\n", ev.Fields)
		if i+1 < len(events) {
			fmt.Println()
		}
	}

	format := "tm[%#v] = reflect.TypeOf(%s%s{})\n"

	for _, ev := range events {
		fmt.Printf(format, [2]string{ev.ModuleName, ev.Name}, ev.ModuleName, ev.Name)
	}

}
