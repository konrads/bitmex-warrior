Want to trade Bitmex like a Warrior?
====================================

Build status (master): [![Build Status](https://travis-ci.org/konrads/bitmex-warrior.svg?branch=master)](https://travis-ci.org/konrads/bitmex-warrior)


![warrior_on_the_moon](doc/image/warrior_on_the_moon.jpg?raw=true)

Get yourself some keyboard shortcuts!

Disclaimer: This is a Rust playground project, I know there are alternatives eg. Tampermonkey.

TODOS
-----
* fix Market response HTTP 400:
STATUS: Received unexpected http response: Response { url: Url { scheme: "https", host: Some(Domain("testnet.bitmex.com")), port: None, path: "/api/v1/order", query: None, fragment: None }, status: 400, headers: {"date": "Thu, 25 Feb 2021 21:48:10 GMT", "content-type": "application/json; charset=utf-8", "content-length": "56", "connection": "keep-alive", "set-cookie": "AWSALBTG=kaj9gVo+EjwyLRTd9aP+wsE4J/Ecd4U6jCMgv+40xgO93DLxUKYCFq/ONiip+FiWtbOCnzkh4iEoKuu13VbsAKW6UoXyNDRrK7w/2Z4c0MrPPyuicMl74yG4fZwSzwBuZTQHXZCmIysP8JRp2sjRrvXhTFEs3hBww5KZPSXsO9HN; Expires=Thu, 04 Mar 2021 21:48:10 GMT; Path=/", "set-cookie": "AWSALBTGCORS=kaj9gVo+EjwyLRTd9aP+wsE4J/Ecd4U6jCMgv+40xgO93DLxUKYCFq/ONiip+FiWtbOCnzkh4iEoKuu13VbsAKW6UoXyNDRrK7w/2Z4c0MrPPyuicMl74yG4fZwSzwBuZTQHXZCmIysP8JRp2sjRrvXhTFEs3hBww5KZPSXsO9HN; Expires=Thu, 04 Mar 2021 21:48:10 GMT; Path=/; SameSite=None; Secure", "x-ratelimit-limit": "60", "x-ratelimit-remaining": "59", "x-ratelimit-reset": "1614289691", "x-ratelimit-remaining-1s": "9", "x-powered-by": "Profit", "etag": "W/\"38-VA4Y4az2ZWIZFNiNZx+BrZha5y4\"", "strict-transport-security": "max-age=31536000; includeSubDomains"} }
* test cancel
* add optargs for: main (config file) and cli (for diff options)
* convert to Result<>
* remove unwrap()s
* remove compile warnings...
* consider integration vs unit tests...
* add doc comment tests
