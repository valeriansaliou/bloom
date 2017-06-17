Bloom
=====

[![Build Status](https://travis-ci.org/valeriansaliou/bloom.svg?branch=master)](https://travis-ci.org/valeriansaliou/bloom) [![Coverage Status](https://coveralls.io/repos/github/valeriansaliou/bloom/badge.svg?branch=master)](https://coveralls.io/github/valeriansaliou/bloom?branch=master)

**:cherry_blossom: Bloom is a REST API caching middleware, acting as a reverse proxy between your load balancers and your REST API workers.**

It is completely agnostic of your API implementation, and requires minimal changes to your existing API code to work.

Bloom relies on `memcached` to store cached data. It is built in Rust and focuses on performance and low resource usage.

**Important: Bloom works great if your API implements REST conventions. Your API needs to use HTTP read methods, namely `GET`, `HEAD`, `OPTIONS` solely as read methods (do not use HTTP GET parameters as a way to update data).**

**🚨 Currently Work In Progress (WIP)**.

![Bloom](https://valeriansaliou.github.io/bloom/images/bloom.jpg)

## Who uses it?

<table>
<tr>
<td align="center"><a href="https://crisp.im/"><img src="https://valeriansaliou.github.io/bloom/images/crisp.png" height="64" /></a></td>
</tr>
<tr>
<td align="center">Crisp</td>
</tr>
</table>

_👋 You use Bloom and you want to be listed there? [Contact me](https://valeriansaliou.name/)!_

## Features

* **Cache stored on buckets**, specified in your REST API responses using HTTP header `Bloom-Strategy-Bucket`.
* **Cache clustered by authentication token**, no cache leak across users is possible, using the standard `Authorization` HTTP header.
* **Cache can be expired directly from your REST API workers** (by hitting against `memcached`).
* **Configurable per-route / per-response caching strategy**, using `Bloom-Strategy-*` HTTP headers in your API responses.
  * Disable all cache for an API route with `Bloom-Strategy-Ignore`.
  * Specify caching bucket for an API route with `Bloom-Strategy-Bucket`.
  * Specify caching TTL in seconds for an API route with `Bloom-Strategy-TTL` (other than default TTL).
* **Serve `304 Not Modified` to non-modified route contents**, lowering bandwidth usage and speeding up requests to your users.

## The Bloom Approach

Bloom can be hot-plugged to sit between your existing Load Balancers (eg. NGINX), and your API workers (eg. NodeJS). It has been initially built to reduce the workload and drastically reduce CPU usage in case of API traffic spike, or DOS / DDoS attacks.

A simpler caching approach could have been to enable caching at the Load Balancer level for HTTP read methods (`GET`, `HEAD`, `OPTIONS`). Although simple as a solution, it would not work with a REST API. REST API serve dynamic content by nature, that rely heavily on Authorization headers. Also, any cache needs to be purged at some point, if the content in cache becomes stale due to data updates in some database.

NGINX Lua scripts could do that job just fine, you say! Well, I firmly believe Load Balancers should be simple, and be based on configuration only, without scripting. As Load Balancers are the entry point to all your HTTP / WebSocket services, you'd want to avoid frequent deployments and custom code there, and handoff that caching complexity to a dedicated middleware component.

## How does it work?

Bloom is installed on the same box as each of your API workers. As seen from your Load Balancers, there is a Bloom instance per API worker. This way, your Load Balancing strategy (eg. Round-Robin with health checks) is not broken. Each Bloom instance can be set to be visible from its own LAN IP your Load Balancers can point to, and then those Bloom instances can point to your API worker listeners on the local loopback.

Bloom acts as a Reverse Proxy of its own, and caches read HTTP methods (`GET`, `HEAD`, `OPTIONS`), while directly proxying HTTP write methods (`POST`, `PATCH`, `PUT` and others). All Bloom instances share the same cache storage on a common `memcached` instance available on the LAN.

Bloom is built in Rust for memory safety, code elegance and especially performance. Bloom can be compiled to native code for your server architecture.

Bloom has minimal static configuration, and relies on HTTP response headers served by your API workers to configure caching on a per-response basis. Those HTTP headers are intercepted by Bloom and not served to your Load Balancer responses. Those headers are formatted as `Bloom-Strategy-*`. Upon serving response to your Load Balancers, Bloom sets a cache status header, namely `Bloom-Status` which can be seen publicly in HTTP responses (either with value `HIT`, `MISS` or `DIRECT` — it helps debug your cache configuration).

![Bloom Schema](https://valeriansaliou.github.io/bloom/docs/models/schema.png)

## How to use it?

* **TODO: install**
* **TODO: configure API**
* **TODO: expiring cache**

## How fast is it?

Bloom is built in Rust, which can be compiled to native code for your architecture. Rust, unlike eg. Golang, doesn't carry a GC (Garbage Collector), which is usually a bad thing for high-throughput / high-load production systems (as a GC halts all program instruction execution for an amount of time that depends on how many references are kept in memory).

Benchmarks are performed and updated upon major code changes, to measure Bloom performance and try to get the highest throughput for the lowest pressure on system resources (CPU / RAM). You can find them below.

**(benchmark coming...)**

## How does it deal with authenticated routes?

Authenticated routes are usually used by REST API to return data that's private to the requester user. Bloom being a cache system, it is critical that no cache leak from an authenticated route occur. Bloom solves the issue easily by isolating cache in namespaces for requests that send an HTTP `Authorization` header. This is the default, secure behavior.

If a route is being requested without HTTP `Authorization` header (ie. the request is anonymous / public), whatever the HTTP response code, that response will be cached by Bloom.

As your HTTP `Authorization` header contains sensitive authentication data (ie. username and password), Bloom stores those values hashed in `memcached` (using a cryptographic hash function). That way, a `memcached` database leak on your side will not allow an attacker to recover authentication key pairs.

## Can cache be programatically expired?

Yes. As your existing API worker performs the database updates on its end, it is already well aware of when data - that might be cached by Bloom - becomes stale. Therefore, Bloom provides an efficient way to tell it to expire cache for a given bucket. This system is called Bloom Control.

Bloom can be configured to listen on a TCP socket to expose a cache control interface. Bloom implements a basic Command-ACK protocol. This way, your API worker (or any other worker in your infrastructure) can tell Bloom to expire cache for a given bucket. Note that as a given bucket may contain variations of cache for different HTTP `Authorization` headers, cache for all authentication tokens is purged at the same time.

**Bloom Control Libraries:**

* **NodeJS**: **TODO**
* 👉 Cannot find the library for your programming language? Build your own and be referenced here! ([contact me](https://valeriansaliou.name/))
