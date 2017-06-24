// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

const METHODS_CACHE: &'static [&'static str] = &[
    "HEAD",
    "GET"
];

const STATUS_CODES_CACHE: &'static [u16] = &[
    // 2xx
    200,  // OK
    203,  // Non-Authoritative Information
    204,  // No Content
    205,  // Reset Content
    206,  // Partial Content
    207,  // Multi-Status
    208,  // Already Reported

    // 3xx
    300,  // Multiple Choices
    301,  // Moved Permanently
    302,  // Found
    303,  // See Other
    308,  // Permanent Redirect

    // 4xx
    401,  // Unauthorized
    402,  // Payment Required
    403,  // Forbidden
    404,  // Not Found
    405,  // Method Not Allowed
    410,  // Gone
    414,  // URI Too Long
    423,  // Locked
    424,  // Failed Dependency

    // 5xx
    501   // Not Implemented
];
