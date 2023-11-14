# Lampo Forward Proxy

Designed to be fast, lightweight and handle high traffic spikes, it's bundled with MongoDB to manage authentication and in-memory cache to optimize speed.

Supported Protocols:

- HTTP
- HTTPs (through CONNECT)
- SOCKS5

Supported SOCKS5 commands:

- CONNECT
- UDP ASSOCIATE

Lampo comes with a pre-load backpressure mechanism to avoid CPU spikes when binding to many sockets at once on program launch, you can set it with the `tasks` option under the `preload` config directive, indicating the maximum parellel sockets to bind at once until all sockets are bound.
