module github.com/micafp/unified-shield/go-bridge

go 1.24.4

require (
	github.com/yggdrasil-network/yggdrasil-go v0.5.8
	golang.org/x/net v0.49.0
)

require (
	github.com/Arceliar/ironwood v0.0.0-20240502173819-a386799f3326 // indirect
	github.com/Arceliar/phony v0.0.0-20221101232838-4c74c74b3bc8 // indirect
	github.com/andybalholm/brotli v1.2.0 // indirect
	github.com/flynn/noise v1.1.0 // indirect
	github.com/gologme/log v1.3.0 // indirect
	github.com/hashicorp/golang-lru/v2 v2.0.7 // indirect
	github.com/kardianos/minwinsvc v1.0.2 // indirect
	github.com/klauspost/compress v1.18.3 // indirect
	github.com/klauspost/cpuid/v2 v2.3.0 // indirect
	github.com/klauspost/reedsolomon v1.13.0 // indirect
	github.com/net2share/vaydns v0.0.0-00010101000000-000000000000 // indirect
	github.com/pkg/errors v0.9.1 // indirect
	github.com/quic-go/quic-go v0.44.0 // indirect
	github.com/refraction-networking/utls v1.8.2 // indirect
	github.com/sirupsen/logrus v1.9.4 // indirect
	github.com/tjfoc/gmsm v1.4.1 // indirect
	github.com/xtaci/kcp-go/v5 v5.6.61 // indirect
	github.com/xtaci/smux v1.5.50 // indirect
	gitlab.torproject.org/tpo/anti-censorship/pluggable-transports/goptlib v1.6.0 // indirect
	golang.org/x/crypto v0.47.0 // indirect
	golang.org/x/mobile v0.0.0-20240520174049-1e9ff8689818 // indirect
	golang.org/x/sys v0.40.0 // indirect
	golang.org/x/text v0.33.0 // indirect
	golang.org/x/time v0.14.0 // indirect
	noizns v0.0.0 // indirect
	vaydns-mobile v0.0.0-00010101000000-000000000000 // indirect
	www.bamsoftware.com/git/dnstt.git v0.0.0-00010101000000-000000000000 // indirect
)

replace (
	github.com/net2share/vaydns => ./vaydns
	github.com/xtaci/kcp-go/v5 => github.com/net2share/kcp-go/v5 v5.0.0-20260325165956-416ba9d3856d
	noizns => ./noizdns
	vaydns-mobile => ./vaydns-mobile
	www.bamsoftware.com/git/dnstt.git => ./dnstt
)
