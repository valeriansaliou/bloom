Bloom
=====

[![Build Status](https://travis-ci.org/valeriansaliou/bloom.svg?branch=master)](https://travis-ci.org/valeriansaliou/bloom)

**:cherry_blossom: Bloom is a REST API caching middleware, acting as a reverse proxy between your load balancers and your REST API workers.**

It is completely agnostic of your API implementation, and requires minimal changes to your existing API code to work.

Bloom relies on `memcached` to store cached data.

**Currently Work In Progress (WIP)**.

## Features

* Cache is stored by buckets, specified in your REST API responses.
* Cache clustered by authentication token, no cache leak across users is possible.
* Cache can be expired directly from your REST API workers.
* Configurable per-route / per-response caching strategy, using `X-Bloom-*` HTTP headers in your API responses.
  * Disable all cache for an API route with `X-Bloom-Ignore`
  * Specify caching bucket for an API route with `X-Bloom-Bucket`
  * Specify caching TTL for an API route with `X-Bloom-TTL` (other than default TTL)
* (more coming...)

## How does it work?

**TODO**
**(schema coming...)**

## How to use?

**TODO**

## How fast is it?

**TODO**
**(benchmark coming...)**

## How does it deal with authenticated routes?

**TODO**

## Who uses it?

<table>
<tr>
<td align="center"><a href="https://crisp.im/"><img src="https://valeriansaliou.github.io/bloom/images/crisp.png" height="64" /></a></td>
</tr>
<tr>
<td align="center">Crisp</td>
</tr>
</table>
