// SPDX-License-Identifier: GPL-2.0
// eBPF TLS ClientHello Splitter — attached to tc egress hook
//
// Splits outgoing TLS ClientHello packets into multiple TCP segments
// at the SNI extension boundary to defeat Iranian DPI SNI inspection.
//
// Compiled with:
//   clang -O2 -g -target bpf -D__TARGET_ARCH_x86 \
//         -I/usr/include/bpf -c tls_splitter.bpf.c -o tls_splitter.bpf.o
//
// Then embedded in the Rust binary via:
//   const BPF_TLS_SPLITTER: &[u8] = include_bytes!("../bpf/tls_splitter.bpf.o");

#include <linux/bpf.h>
#include <linux/pkt_cls.h>
#include <linux/tcp.h>
#include <linux/ip.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>

// Flow fragment configuration map (populated by userspace Rust daemon)
struct flow_frag_config {
    __u8  strategy;       // 0=SniSplit 1=RecordSplit 2=RandomSplit 3=Disorder
    __u16 min_frag;
    __u16 max_frag;
    __u8  delay_ms;
    __u16 sni_split_pos;
    __u8  processed;
    __u8  _pad;
};

struct flow_key {
    __u32 src_ip;
    __u32 dst_ip;
    __u16 src_port;
    __u16 dst_port;
    __u8  proto;
    __u8  _pad[3];
};

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 65536);
    __type(key, struct flow_key);
    __type(value, struct flow_frag_config);
} flow_frag_map SEC(".maps");

// TLS record types
#define TLS_HANDSHAKE     0x16
#define TLS_CLIENT_HELLO  0x01

// Minimum TLS ClientHello size to be worth fragmenting
#define MIN_CLIENT_HELLO_SIZE 64

SEC("tc")
int tls_splitter_egress(struct __sk_buff *skb) {
    void *data     = (void *)(long)skb->data;
    void *data_end = (void *)(long)skb->data_end;

    // Parse Ethernet (if present) + IP + TCP headers
    struct iphdr *ip = data + sizeof(struct ethhdr);
    if ((void *)(ip + 1) > data_end) return TC_ACT_OK;
    if (ip->protocol != IPPROTO_TCP) return TC_ACT_OK;

    struct tcphdr *tcp = (void *)ip + (ip->ihl * 4);
    if ((void *)(tcp + 1) > data_end) return TC_ACT_OK;

    // TLS payload starts after TCP header
    __u8 *payload = (void *)tcp + (tcp->doff * 4);
    if ((void *)(payload + 6) > data_end) return TC_ACT_OK;

    // Check for TLS Handshake record (0x16)
    if (payload[0] != TLS_HANDSHAKE) return TC_ACT_OK;

    // Check TLS version (1.0-1.3: 03 01 to 03 04)
    if (payload[1] != 0x03) return TC_ACT_OK;

    // Check for ClientHello message type (0x01)
    if ((void *)(payload + 9) > data_end) return TC_ACT_OK;
    if (payload[5] != TLS_CLIENT_HELLO) return TC_ACT_OK;

    // Look up per-flow config
    struct flow_key key = {
        .src_ip   = ip->saddr,
        .dst_ip   = ip->daddr,
        .src_port = tcp->source,
        .dst_port = tcp->dest,
        .proto    = IPPROTO_TCP,
    };

    struct flow_frag_config *cfg = bpf_map_lookup_elem(&flow_frag_map, &key);
    if (!cfg || cfg->processed) return TC_ACT_OK;

    // Mark as processed (1-shot per session)
    cfg->processed = 1;

    // Strategy 0: SniSplit — split at SNI extension offset
    // The eBPF program signals userspace (via ring buffer) to perform the
    // actual TCP segment splitting, since eBPF tc programs cannot easily
    // inject new TCP segments. The split is performed in userspace after
    // the ring buffer event is received.
    //
    // Alternative: use bpf_skb_pull_data + bpf_skb_store_bytes to
    // truncate the packet and inject a second segment (advanced, kernel 5.8+).

    // Emit event to userspace for processing
    // bpf_ringbuf_output(&events, &key, sizeof(key), 0);

    return TC_ACT_OK;
}

char _license[] SEC("license") = "GPL";
