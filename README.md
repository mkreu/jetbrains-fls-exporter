# Prometheus Exporter for JetBrains Floating License Server

This is a small utility that exports
[real-time usage statistics from a JetBrains license server](https://www.jetbrains.com/help/license_server/detailed_server_usage_statistics.html#d7f5f0fa)
as prometheus metrics.

## Published Metrics

```
# HELP jls_licenses_allocated Number of JLS Licenses currently allocated
# TYPE jls_licenses_allocated gauge
jls_licenses_allocated{license_name="Rider"} 3
jls_licenses_allocated{license_name="CLion"} 5
# HELP jls_licenses_available Number of JLS Licenses currently available
# TYPE jls_licenses_available gauge
jls_licenses_available{license_name="Rider"} 7
jls_licenses_available{license_name="CLion"} 5
```

## How to run

### Using Docker (or Podman)

```sh
# build the container
docker build --tag jls-exporter .

# run
docker run -d \
        --name jls-exporter \
        -p 9836:9836
        -e JLS_BASE_URL="https://example.com:8080" \
        -e JLS_STATS_TOKEN="<supersecrettoken>" \
        --stop-timeout 1 \
        jls-exporter

```

### Without a container

Install [Rust](https://rustup.rs), set the environment variables and use
`cargo run --release`

## Environment Variables

Variable | Default Value | Explanation
-------- | ------------- | ------------
JLS_BASE_URL | None, always required | Base URL of your license server
JLS_STATS_TOKEN | None, always required | [API token of your license server](https://www.jetbrains.com/help/license_server/detailed_server_usage_statistics.html#7ad5d2e6)
JLS_EXPORTER_BINDADDR | `0.0.0.0:9836` | Default address this exporter should bind to
