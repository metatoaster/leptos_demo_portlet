# Portlet Example in Leptos

An example to demostrate reusable portlets via effects and signals in
Leptos.  The way to reuse them is to provide the expected data to the
relevant signals.

## Quick Start

This demo is implemented in a way that integrates both axum and actix
as options that may be toggled using ``--bin-features`` flag.

Run:

- `cargo leptos watch --bin-features axum` to serve using axum.
- `cargo leptos watch --bin-features actix` to serve using actix.
