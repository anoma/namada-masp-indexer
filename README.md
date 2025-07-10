# Namada Masp Indexer

The [`namada-masp-indexer`](https://github.com/namada-net/namada-masp-indexer) is a
specialized indexer that crawls [Namada](https://github.com/anoma/namada)
networks, extracting [MASP](https://github.com/anoma/masp) transaction data. In
addition to indexing fetched MASP transactions, the `namada-masp-indexer` builds
a panoply of data structures that keep track of the state of the current MASP
commitment tree, note positions, etc. By exposing this data via an HTTP RPC API,
Namada clients are able to synchronize with the latest state of the MASP very
quickly, alleviating remote procedure calls to full nodes.

## Status

⚠️ This project is still a work-in-progress, use at your own risk! ⚠️

## How to run

- Copy the `.env.template` to `.env` file and edit the necessary variables.
- Run `docker compose up`

## License

This project is licensed under the GNU General Public License v3.0. You can
consult a copy of the license text [here](COPYING).
