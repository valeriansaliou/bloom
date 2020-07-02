Bloom
=====

[![Build Status](https://travis-ci.org/valeriansaliou/bloom.svg?branch=master)](https://travis-ci.org/valeriansaliou/bloom) [![Buy Me A Coffee](https://img.shields.io/badge/buy%20me%20a%20coffee-donate-yellow.svg)](https://www.buymeacoffee.com/valeriansaliou)

**Bloom is a REST API caching middleware, acting as a reverse proxy between your load balancers and your REST API workers.**

It is completely agnostic of your API implementation, and requires minimal changes to your existing API code to work.

Bloom relies on `redis`, [configured as a cache](https://github.com/valeriansaliou/bloom/blob/master/examples/config/redis.conf) to store cached data. It is built in Rust and focuses on stability, performance and low resource usage.

**Important: Bloom works great if your API implements REST conventions. Your API needs to use HTTP read methods, namely `GET`, `HEAD`, `OPTIONS` solely as read methods (do not use HTTP GET parameters as a way to update data).**

_Tested at Rust version: `rustc 1.43.0 (4fb7144ed 2020-04-20)`_

**🇫🇷 Crafted in Brest, France.**

**:newspaper: The Bloom project was initially announced in [a post on my personal journal](https://journal.valeriansaliou.name/announcing-bloom-a-rest-api-caching-middleware/).**

![Bloom](https://valeriansaliou.github.io/bloom/images/bloom.jpg)

## Who uses it?

<table>
<tr>
<td align="center"><a href="https://crisp.chat/"><img src="https://valeriansaliou.github.io/bloom/images/crisp.png" height="64" /></a></td>
</tr>
<tr>
<td align="center">Crisp</td>
</tr>
</table>

_👋 You use Bloom and you want to be listed there? [Contact me](https://valeriansaliou.name/)._

## Features

* **The same Bloom server can be used for different API workers at once**, using HTTP header `Bloom-Request-Shard` (eg. Main API uses shard `0`, Search API uses shard `1`).
* **Cache stored on buckets**, specified in your REST API responses using HTTP header `Bloom-Response-Buckets`.
* **Cache clustered by authentication token**, no cache leak across users is possible, using the standard `Authorization` HTTP header.
* **Cache can be expired directly from your REST API workers**, via a control channel.
* **Configurable per-request caching strategy**, using `Bloom-Request-*` HTTP headers in the requests your Load Balancers forward to Bloom.
  * Specify caching shard for an API system with `Bloom-Request-Shard` (default shard is `0`, maximum value is `15`).
* **Configurable per-response caching strategy**, using `Bloom-Response-*` HTTP headers in your API responses to Bloom.
  * Disable all cache for an API route with `Bloom-Response-Ignore` (with value `1`).
  * Specify caching buckets for an API route with `Bloom-Response-Buckets` (comma-separated if multiple buckets).
  * Specify caching TTL in seconds for an API route with `Bloom-Response-TTL` (other than default TTL, number in seconds).
* **Serve `304 Not Modified` to non-modified route contents**, lowering bandwidth usage and speeding up requests to your users.

## The Bloom Approach

Bloom can be hot-plugged to sit between your existing Load Balancers (eg. NGINX), and your API workers (eg. NodeJS). It has been initially built to reduce the workload and drastically reduce CPU usage in case of API traffic spike, or DOS / DDoS attacks.

A simpler caching approach could have been to enable caching at the Load Balancer level for HTTP read methods (`GET`, `HEAD`, `OPTIONS`). Although simple as a solution, it would not work with a REST API. REST API serve dynamic content by nature, that rely heavily on Authorization headers. Also, any cache needs to be purged at some point, if the content in cache becomes stale due to data updates in some database.

NGINX Lua scripts could do that job just fine, you say! Well, I firmly believe Load Balancers should be simple, and be based on configuration only, without scripting. As Load Balancers are the entry point to all your HTTP / WebSocket services, you'd want to avoid frequent deployments and custom code there, and handoff that caching complexity to a dedicated middleware component.

## How does it work?

Bloom is installed on the same server as each of your API workers. As seen from your Load Balancers, there is a Bloom instance per API worker. This way, your Load Balancing setup (eg. Round-Robin with health checks) is not broken. Each Bloom instance can be set to be visible from its own LAN IP your Load Balancers can point to, and then those Bloom instances can point to your API worker listeners on the local loopback.

Bloom acts as a Reverse Proxy of its own, and caches read HTTP methods (`GET`, `HEAD`, `OPTIONS`), while directly proxying HTTP write methods (`POST`, `PATCH`, `PUT` and others). All Bloom instances share the same cache storage on a common `redis` instance available on the LAN.

Bloom is built in Rust for memory safety, code elegance and especially performance. Bloom can be compiled to native code for your server architecture.

Bloom has minimal static configuration, and relies on HTTP response headers served by your API workers to configure caching on a per-response basis. Those HTTP headers are intercepted by Bloom and not served to your Load Balancer responses. Those headers are formatted as `Bloom-Response-*`. Upon serving response to your Load Balancers, Bloom sets a cache status header, namely `Bloom-Status` which can be seen publicly in HTTP responses (either with value `HIT`, `MISS` or `DIRECT` — it helps debug your cache configuration).

![Bloom Schema](https://valeriansaliou.github.io/bloom/docs/models/schema.png)

## How to use it?

### Installation

Bloom is built in Rust. To install it, either download a version from the [Bloom releases](https://github.com/valeriansaliou/bloom/releases) page, use `cargo install` or pull the source code from `master`.

**Install from source:**

If you pulled the source code from Git, you can build it using `cargo`:

```bash
cargo build --release
```

You can find the built binaries in the `./target/release` directory.

**Install from Cargo:**

You can install Bloom directly with `cargo install`:

```bash
cargo install bloom-server
```

Ensure that your `$PATH` is properly configured to source the Crates binaries, and then run Bloom using the `bloom` command.

**Install from packages:**

Debian & Ubuntu packages are also available. Refer to the _[How to install it on Debian & Ubuntu?](#how-to-install-it-on-debian--ubuntu)_ section.

**Install from Docker Hub:**

You might find it convenient to run Bloom via Docker. You can find the pre-built Bloom image on Docker Hub as [valeriansaliou/bloom](https://hub.docker.com/r/valeriansaliou/bloom/).

First, pull the `valeriansaliou/bloom` image:

```bash
docker pull valeriansaliou/bloom:v1.29.0
```

Then, seed it a configuration file and run it (replace `/path/to/your/bloom/config.cfg` with the path to your configuration file):

```bash
docker run -p 8080:8080 -p 8811:8811 -v /path/to/your/bloom/config.cfg:/etc/bloom.cfg valeriansaliou/bloom:v1.29.0
```

In the configuration file, ensure that:

* `server.inet` is set to `0.0.0.0:8080` (this lets Bloom be reached from outside the container)
* `control.inet` is set to `0.0.0.0:8811` (this lets Bloom Control be reached from outside the container)

Bloom will be reachable from `http://localhost:8080`, and Bloom Control will be reachable from `tcp://localhost:8811`.

### Configuration

Use the sample [config.cfg](https://github.com/valeriansaliou/bloom/blob/master/config.cfg) configuration file and adjust it to your own environment.

Make sure to properly configure the `[proxy]` section so that Bloom points to your API worker host and port.

**Available configuration options are commented below, with allowed values:**

**[server]**

* `log_level` (type: _string_, allowed: `debug`, `info`, `warn`, `error`, default: `error`) — Verbosity of logging, set it to `error` in production
* `inet` (type: _string_, allowed: IPv4 / IPv6 + port, default: `[::1]:8080`) — Host and TCP port the Bloom server should listen on

**[control]**

* `inet` (type: _string_, allowed: IPv4 / IPv6 + port, default: `[::1]:8811`) — Host and TCP port Bloom Control should listen on
* `tcp_timeout` (type: _integer_, allowed: seconds, default: `300`) — Timeout of idle/dead client connections to Bloom Control

**[proxy]**

**[[proxy.shard]]**

* `shard` (type: _integer_, allowed: `0` to `15`, default: `0`) — Shard index (routed using `Bloom-Request-Shard` in requests to Bloom)
* `host` (type: _string_, allowed: hostname, IPv4, IPv6, default: `localhost`) — Target host to proxy to for this shard (ie. where the API listens)
* `port` (type: _integer_, allowed: TCP port, default: `3000`) — Target TCP port to proxy to for this shard (ie. where the API listens)

**[cache]**

* `ttl_default` (type: _integer_, allowed: seconds, default: `600`) — Default cache TTL in seconds, when no `Bloom-Response-TTL` provided
* `executor_pool` (type: _integer_, allowed: `0` to `(2^16)-1`, default: `16`) — Cache executor pool size (how many cache requests can execute at the same time)
* `disable_read` (type: _boolean_, allowed: `true`, `false`, default: `false`) — Whether to disable cache reads (useful for testing)
* `disable_write` (type: _boolean_, allowed: `true`, `false`, default: `false`) — Whether to disable cache writes (useful for testing)
* `compress_body` (type: _boolean_, allowed: `true`, `false`, default: `true`) — Whether to compress body upon store (using Brotli; usually reduces body size by 40%)

**[redis]**

* `host` (type: _string_, allowed: hostname, IPv4, IPv6, default: `localhost`) — Target Redis host
* `port` (type: _integer_, allowed: TCP port, default: `6379`) — Target Redis TCP port
* `password` (type: _string_, allowed: password values, default: none) — Redis password (if no password, dont set this key)
* `database` (type: _integer_, allowed: `0` to `255`, default: `0`) — Target Redis database
* `pool_size` (type: _integer_, allowed: `0` to `(2^32)-1`, default: `80`) — Redis connection pool size (should be a bit higher than `cache.executor_pool`, as it is used by both Bloom proxy and Bloom Control)
* `max_lifetime_seconds` (type: _integer_, allowed: seconds, default: `60`) — Maximum lifetime of a connection to Redis (you want it below 5 minutes, as this affects the reconnect delay to Redis if a connection breaks)
* `idle_timeout_seconds` (type: _integer_, allowed: seconds, default: `600`) — Timeout of idle/dead pool connections to Redis
* `connection_timeout_seconds` (type: _integer_, allowed: seconds, default: `1`) — Timeout in seconds to consider Redis dead and emit a `DIRECT` connection to API without using cache (keep this low, as when Redis is down it dictates how much time to wait before ignoring Redis response and proxying directly)
* `max_key_size` (type: _integer_, allowed: bytes, default: `256000`) — Maximum data size in bytes to store in Redis for a key (safeguard to prevent very large responses to be cached)
* `max_key_expiration` (type: _integer_, allowed: seconds, default: `2592000`) — Maximum TTL for a key cached in Redis (prevents erroneous `Bloom-Response-TTL` values)

### Run Bloom

Bloom can be run as such:

`./bloom -c /path/to/config.cfg`

**Important: make sure to spin up a Bloom instance for each API worker running on your infrastructure. Bloom does not manage the Load Balancing logic itself, so you should have a Bloom instance per API worker instance and still rely on eg. NGINX for Load Balancing.**

### Configure Load Balancers

Once Bloom is running and points to your API, you can configure your Load Balancers to point to Bloom IP and port (instead of your API IP and port as previously).

#### NGINX instructions

**➡️ Configure your existing proxy ruleset**

Bloom requires the `Bloom-Request-Shard` HTTP header to be set by your Load Balancer upon proxying a client request to Bloom. This header tells Bloom which cache shard to use for storing data (this way, you can have a single Bloom instance for different API sub-systems listening on the same server).

```
# Your existing ruleset goes here
proxy_pass http://(...)

# Adds the 'Bloom-Request-Shard' header for Bloom
proxy_set_header Bloom-Request-Shard 0;
```

**➡️ Adjust your existing CORS rules (if used)**

If your API runs on a dedicated hostname (eg. `https://api.crisp.chat` for [Crisp](https://crisp.chat/en/)), do not forget to adjust your [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) rules accordingly, so that API Web clients (ie. browsers) can leverage the [ETag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag) header that gets added by Bloom. This will help speed up API read requests on slower networks. **_If you don't have existing CORS rules, you may not need them, so ignore this._**

```
# Merge those headers with your existing CORS rules
add_header 'Access-Control-Allow-Headers' 'If-Match, If-None-Match' always;
add_header 'Access-Control-Expose-Headers' 'Vary, ETag' always;
```

_Note that a shard number is an integer from 0 to 15 (8-bit unsigned number, capped to 16 shards)._

**The response headers that get added by Bloom are:**

* **ETag**: unique identifier for the response data being returned (enables browser caching); [see MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag).
* **Vary**: tells other cache layers (eg. proxies) that the ETag field may vary on each request, so they need to revalidate it; [see MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Vary).

**The request headers that get added by the browser, as a consequence of Bloom adding the request headers above are:**

* **If-Match**: used by the client to match a given server ETag field (on write requests); [see MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Match).
* **If-None-Match**: used by the client to match a given server ETag field (on read requests); [see MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-None-Match).

_Note that you need to add both new request and response headers to your CORS rules. If you forget either one, requests to your API may start to fail on certain browsers (eg. Chrome with `PATCH` requests)._

### Configure Your API

Now that Bloom is running in front of your API and serving requests on behalf of it; your API can instruct Bloom how to behave on a per-response basis.

Your API can send private HTTP headers in responses to Bloom, that are used by Bloom and removed from the response that is served to the request client (the `Bloom-Response-*` HTTP headers).

_Note that your API should not serve responses in a compressed format. Please disable any Gzip or Brotli middleware on your application server, as Bloom will not be able to decode compressed response bodies. Compression of dynamic content should be handled by the load balancer itself._

**➡️ Do not cache response:**

To tell Bloom not to cache a response, send the following HTTP header as part of the API response:

`Bloom-Response-Ignore: 1`

By default, Bloom retains all responses that are safe to cache, as long as they match both:

**1. Cacheable methods:**

* `GET`
* `HEAD`
* `OPTIONS`

**2. Cacheable status:**

* `OK`
* `Non-Authoritative Information`
* `No Content`
* `Reset Content`
* `Partial Content`
* `Multi-Status`
* `Already Reported`
* `Multiple Choices`
* `Moved Permanently`
* `Found`
* `See Other`
* `Permanent Redirect`
* `Unauthorized`
* `Payment Required`
* `Forbidden`
* `Not Found`
* `Method Not Allowed`
* `Gone`
* `URI Too Long`
* `Unsupported Media Type`
* `Range Not Satisfiable`
* `Expectation Failed`
* `I'm A Teapot`
* `Locked`
* `Failed Dependency`
* `Precondition Required`
* `Request Header Fields Too Large`
* `Not Implemented`
* `Not Extended`

_Refer to [the list of status codes on Wikipedia](https://en.wikipedia.org/wiki/List_of_HTTP_status_codes) if you want to find the matching status codes._

**➡️ Set an expiration time on response cache:**

To tell Bloom to use a certain expiration time on response cache (time after which the cache is invalidated and thus a new response is fetched upon client request), send the following HTTP header as part of the API response (here for a TTL of 60 seconds):

`Bloom-Response-TTL: 60`

By default, Bloom sets a TTL of 600 seconds (10 minutes), though this can be configured from `config.cfg`.

**➡️ Tag a cached response (for Bloom Control cache purge):**

If you'd like to use Bloom Control to programatically purge cached responses (see _[Can cache be programatically expired?](#can-cache-be-programatically-expired)_), you will need to tag those responses when they get cached. You can tell Bloom to tag a cached response in 1 or more bucket, as such:

`Bloom-Response-Buckets: user_id:10012, heavy_route:1203`

Then, when you need to purge the tagged responses for user with identifier `10012`, you can call a Bloom Control cache purge on bucket `user_id:10012`. The flow is similar for bucket `heavy_route:1203`.

By default, a cached response has no tag, thus it cannot be purged via Bloom Control _as-is_.

## How to install it on Debian & Ubuntu?

Bloom provides [pre-built packages](https://packagecloud.io/valeriansaliou/bloom) for Debian-based systems (Debian, Ubuntu, etc.).

**Important: Bloom only provides Debian 8 64 bits packages for now (Debian Buster). You will still be able to use them on other Debian versions, as well as Ubuntu.**

**1️⃣ Add the Bloom APT repository (eg. for Debian Buster):**

```bash
echo "deb https://packagecloud.io/valeriansaliou/bloom/debian/ buster main" > /etc/apt/sources.list.d/valeriansaliou_bloom.list
curl -L https://packagecloud.io/valeriansaliou/bloom/gpgkey 2> /dev/null | apt-key add - &>/dev/null
apt-get update
```

**2️⃣ Install the Bloom package:**

```bash
apt-get install bloom
```

**3️⃣ Edit the pre-filled Bloom configuration file:**

```bash
nano /etc/bloom.cfg
```

**4️⃣ Restart Bloom:**

```
service bloom restart
```

## How fast & lightweight is it?

Bloom is built in Rust, which can be compiled to native code for your architecture. Rust, unlike eg. Golang, doesn't carry a GC (Garbage Collector), which is usually a bad thing for high-throughput / high-load production systems (as a GC halts all program instruction execution for an amount of time that depends on how many references are kept in memory).

Note that some compromises have been made relative to how Bloom manages memory. Heap-allocated objects are heavily used for the sake of simplicify. ie. responses from your API workers are fully buffered in memory before they are served to the client; which has the benefit of draining data from your API workers as fast as your loopback / LAN goes, even if the requester client has a very slow bandwidth.

In production at [Crisp](https://crisp.chat/en/), we're running multiple Bloom instances (for each of our API worker). Each one handles ~250 HTTP RPS (Requests Per Second), as well as ~500 Bloom Control RPS (eg. cache purges). Each Bloom instance runs on a single 2016 Xeon vCPU paired with 512MB RAM. The kind of HTTP requests Bloom handles is balanced between reads (`GET`, `HEAD`, `OPTIONS`) and writes (`POST`, `PATCH`, `PUT` and others).

We get the following `htop` feedback on a server running Bloom at such load:

![htop](https://valeriansaliou.github.io/bloom/images/htop.png)

**As you can see, Bloom consumes only a fraction of the CPU time (less than 5%) for a small RAM footprint (~5% which is ~25MB)**. On such a small server, we can predict Bloom could scale to even higher rates (eg. 10k RPS) without putting too much pressure on the system (the underlying NodeJS API worker would be overheating first as it's much heavier than Bloom).

If you want Bloom to handle very high RPS, make sure to adjust the `cache.executor_pool` and the `redis.pool_size` options to higher values (which may limit your RPS if you have a few milliseconds of latency on your Redis link — as Redis connections are blocking).

## How does it deal with authenticated routes?

Authenticated routes are usually used by REST API to return data that's private to the requester user. Bloom being a cache system, it is critical that no cache leak from an authenticated route occur. Bloom solves the issue easily by isolating cache in namespaces for requests that send an HTTP `Authorization` header. This is the default, secure behavior.

If a route is being requested without HTTP `Authorization` header (ie. the request is anonymous / public), whatever the HTTP response code, that response will be cached by Bloom.

As your HTTP `Authorization` header contains sensitive authentication data (ie. username and password), Bloom stores those values hashed in `redis` (using a cryptographic hash function). That way, a `redis` database leak on your side will not allow an attacker to recover authentication key pairs.

## Can cache be programatically expired?

Yes. As your existing API workers perform the database updates on their end, they are already well aware of when data - _that might be cached by Bloom_ - gets stale. Therefore, Bloom provides an efficient way to tell it to expire cache for a given bucket. This system is called **Bloom Control**.

Bloom can be configured to listen on a TCP socket to expose a cache control interface. The default TCP port is 8811. Bloom implements a basic Command-ACK protocol.

This way, your API worker (or any other worker in your infrastructure) can either tell Bloom to:

* **Expire cache for a given bucket.** Note that as a given bucket may contain variations of cache for different HTTP `Authorization` headers, bucket cache for all authentication tokens is purged at the same time when you purge cache for a bucket.
* **Expire cache for a given HTTP `Authorization` header.** Useful if an user logs-out and revokes their authentication token.

**➡️ Available commands:**

* `FLUSHB <namespace>`: flush cache for given bucket namespace
* `FLUSHA <authorization>`: flush cache for given authorization
* `SHARD <shard>`: select shard to use for connection
* `PING`: ping server
* `QUIT`: stop connection

**⬇️ Control flow example:**

```bash
telnet bloom.local 8811
Trying ::1...
Connected to bloom.local.
Escape character is '^]'.
CONNECTED <bloom v1.0.0>
HASHREQ hxHw4AXWSS
HASHRES 753a5309
STARTED
SHARD 1
OK
FLUSHB 2eb6c00c
OK
FLUSHA b44c6f8e
OK
PING
PONG
QUIT
ENDED quit
Connection closed by foreign host.
```

**Notice: before any command can be issued, Bloom requires the client to validate its hasher function against the Bloom internal hasher (done with the `HASHREQ` and `HASHRES` exchange). FarmHash is used to hash keys, using the FarmHash.fingerprint32(), which computed results may vary between architectures. This way, most weird Bloom Control issues are prevented in advance.**

**📦 Bloom Control Libraries:**

* **NodeJS**: **[node-bloom-control](https://www.npmjs.com/package/bloom-control)**

👉 Cannot find the library for your programming language? Build your own and be referenced here! ([contact me](https://valeriansaliou.name/))

## :fire: Report A Vulnerability

If you find a vulnerability in Bloom, you are more than welcome to report it directly to [@valeriansaliou](https://github.com/valeriansaliou) by sending an encrypted email to [valerian@valeriansaliou.name](mailto:valerian@valeriansaliou.name). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production servers running an unpatched Bloom instance.

**:warning: You must encrypt your email using [@valeriansaliou](https://github.com/valeriansaliou) GPG public key: [:key:valeriansaliou.gpg.pub.asc](https://valeriansaliou.name/files/keys/valeriansaliou.gpg.pub.asc).**

**:gift: Based on the severity of the vulnerability, I may offer a $200 (US) bounty to whomever reported it.**
