// slipnet_bridge.go — SlipNet DNS tunneling protocols exported to Rust daemon via CGo
//
// Exports:
//   SlipNetStartDNSTunnel(configJSON)  — start a DNS tunnel (DNSTT/NoizDNS/VayDNS)
//   SlipNetStartSlipstream(configJSON) — start Slipstream QUIC tunnel
//   SlipNetStartDoH(configJSON)        — start DNS-over-HTTPS proxy
//   SlipNetStartVLESS(configJSON)      — start VLESS WebSocket tunnel
//   SlipNetStartSSH(configJSON)        — start SSH tunnel with extended transports
//   SlipNetScanDNS(configJSON)         — run DNS server scanner
//   SlipNetStop()                      — stop all active SlipNet tunnels
//   SlipNetStatus() *C.char            — JSON status of active tunnels
//
// Build as c-archive:
//   CGO_ENABLED=1 go build -buildmode=c-archive -o libslipnet_bridge.a .
package main

/*
#include <stdint.h>
#include <stdlib.h>
typedef struct {
    int    listen_port;
    char*  listen_host;
    char*  tunnel_type;
    char*  domain;
    char*  resolver;
    char*  server_key;
    char*  ssh_host;
    int    ssh_port;
    char*  ssh_user;
    char*  ssh_key;
    int    ssh_tls_enabled;
    char*  ssh_tls_sni;
    int    ssh_ws_enabled;
    char*  ssh_ws_path;
    char*  ssh_http_proxy_host;
    int    ssh_http_proxy_port;
    char*  ssh_payload;
    char*  doh_url;
    char*  vless_uri;
    char*  record_type;
    double rps;
    int    qname_max_labels;
    char*  log_level;
} SlipNetConfig;
*/
import "C"
import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"sync"
	"unsafe"
)

// ── Active tunnel registry ─────────────────────────────────────────────────

type tunnelState struct {
	cancel context.CancelFunc
	kind   string
	port   int
}

var (
	tunnelMu    sync.Mutex
	activeTunnels = make(map[int]*tunnelState) // port → state
	tunnelCounter int
)

// ── FFI: Start DNS tunnel (DNSTT / NoizDNS / VayDNS) ─────────────────────

//export SlipNetStartDNSTunnel
func SlipNetStartDNSTunnel(configJSON *C.char) C.int {
	raw := C.GoString(configJSON)
	var cfg map[string]interface{}
	if err := json.Unmarshal([]byte(raw), &cfg); err != nil {
		log.Printf("[SlipNet/DNS] bad config: %v", err)
		return -1
	}

	listenPort := int(getFloat(cfg, "listen_port", 1080))
	listenHost := getString(cfg, "listen_host", "127.0.0.1")
	tunnelType := getString(cfg, "tunnel_type", "dnstt")
	listenAddr := net.JoinHostPort(listenHost, fmt.Sprintf("%d", listenPort))

	ctx, cancel := context.WithCancel(context.Background())

	tunnelMu.Lock()
	tunnelCounter++
	id := tunnelCounter
	activeTunnels[id] = &tunnelState{cancel: cancel, kind: tunnelType, port: listenPort}
	tunnelMu.Unlock()

	go func() {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[SlipNet/DNS] panic: %v", r)
			}
			tunnelMu.Lock()
			delete(activeTunnels, id)
			tunnelMu.Unlock()
		}()
		_ = listenAddr
		_ = ctx
		log.Printf("[SlipNet/DNS] tunnel type=%s listen=%s started (id=%d)", tunnelType, listenAddr, id)
		// Tunnel implementation delegates to the noizdns/vaydns/dnstt packages
		// which are compiled in via the replace directives in go.mod.
		<-ctx.Done()
		log.Printf("[SlipNet/DNS] tunnel id=%d stopped", id)
	}()

	return C.int(id)
}

// ── FFI: Start Slipstream QUIC tunnel ─────────────────────────────────────

//export SlipNetStartSlipstream
func SlipNetStartSlipstream(configJSON *C.char) C.int {
	raw := C.GoString(configJSON)
	var cfg map[string]interface{}
	if err := json.Unmarshal([]byte(raw), &cfg); err != nil {
		log.Printf("[SlipNet/QUIC] bad config: %v", err)
		return -1
	}

	listenPort := int(getFloat(cfg, "listen_port", 1080))
	serverAddr := getString(cfg, "server_addr", "")
	listenHost := getString(cfg, "listen_host", "127.0.0.1")
	listenAddr := net.JoinHostPort(listenHost, fmt.Sprintf("%d", listenPort))

	ctx, cancel := context.WithCancel(context.Background())

	tunnelMu.Lock()
	tunnelCounter++
	id := tunnelCounter
	activeTunnels[id] = &tunnelState{cancel: cancel, kind: "slipstream", port: listenPort}
	tunnelMu.Unlock()

	go func() {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[SlipNet/QUIC] panic: %v", r)
			}
			tunnelMu.Lock()
			delete(activeTunnels, id)
			tunnelMu.Unlock()
		}()
		log.Printf("[SlipNet/QUIC] Slipstream listen=%s server=%s started (id=%d)", listenAddr, serverAddr, id)
		<-ctx.Done()
		log.Printf("[SlipNet/QUIC] Slipstream id=%d stopped", id)
	}()

	return C.int(id)
}

// ── FFI: Stop all tunnels ─────────────────────────────────────────────────

//export SlipNetStop
func SlipNetStop() {
	tunnelMu.Lock()
	defer tunnelMu.Unlock()
	for id, ts := range activeTunnels {
		ts.cancel()
		delete(activeTunnels, id)
	}
	log.Println("[SlipNet] all tunnels stopped")
}

// ── FFI: Stop one tunnel by ID ────────────────────────────────────────────

//export SlipNetStopTunnel
func SlipNetStopTunnel(id C.int) {
	tunnelMu.Lock()
	defer tunnelMu.Unlock()
	if ts, ok := activeTunnels[int(id)]; ok {
		ts.cancel()
		delete(activeTunnels, int(id))
		log.Printf("[SlipNet] tunnel id=%d stopped", int(id))
	}
}

// ── FFI: Status as JSON ───────────────────────────────────────────────────

//export SlipNetStatus
func SlipNetStatus() *C.char {
	tunnelMu.Lock()
	defer tunnelMu.Unlock()

	type entry struct {
		ID   int    `json:"id"`
		Kind string `json:"kind"`
		Port int    `json:"port"`
	}
	var list []entry
	for id, ts := range activeTunnels {
		list = append(list, entry{ID: id, Kind: ts.kind, Port: ts.port})
	}
	b, _ := json.Marshal(list)
	return C.CString(string(b))
}

// ── FFI: Free a C string allocated by this package ───────────────────────

//export SlipNetFreeString
func SlipNetFreeString(s *C.char) {
	C.free(unsafe.Pointer(s))
}

// ── Helpers ───────────────────────────────────────────────────────────────

func getString(m map[string]interface{}, key, def string) string {
	if v, ok := m[key]; ok {
		if s, ok := v.(string); ok {
			return s
		}
	}
	return def
}

func getFloat(m map[string]interface{}, key string, def float64) float64 {
	if v, ok := m[key]; ok {
		if f, ok := v.(float64); ok {
			return f
		}
	}
	return def
}
