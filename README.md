# 🟡 Namada MASP Indexer

## Status

- 🔧 - This project is a work in progress. 
- 🚧 - Functionality is not guaranteed at this stage. 
- ⚠️ - Use at your own risk.

##  About 

This repository, **Namada MASP Indexer**, is distinct from and incomparable to the similarly named [Namada Indexer](https://github.com/anoma/namada-indexer).

Note that: `Namada Indexer != Namada MASP Indexer && Namada MASP Indexer != Namada Indexer`.

The **Namada MASP Indexer** is a specialized indexer that crawls [Namada](https://github.com/anoma/namada) networks, extracting [MASP](https://github.com/anoma/masp) transaction data. In addition to indexing fetched MASP transactions, the Namada MASP Indexer builds a panoply of data structures that keep track of the state of the current MASP commitment tree, note positions, etc. By exposing this data via an HTTP RPC API, Namada clients are able to synchronize with the latest state of the MASP very quickly, alleviating remote procedure calls to full nodes.

# 🚀 Getting Started

Follow these instructions to set up the project locally. The steps below will guide you through the process of getting a local copy up and running.

It is strongly recommended to change the default username and password for your PostgreSQL database for security purposes. Update these credentials in the `docker-compose.yml` file.

## 🐳 Docker Deployment

### Prerequisites

Before starting, ensure you have the necessary tools and dependencies installed. Below are the steps to set up the required environment.

- **Packages**: Install prerequisite packages from the APT repository.

```sh
apt-get install -y curl apt-transport-https ca-certificates software-properties-common git nano just build-essential
```

- **Docker**: Follow the official instructions provided by Docker to install it: [Install Docker Engine](https://docs.docker.com/engine/install/).

### Usage
Ensure you have the latest repository cloned to maintain compatibility with other Namada interfaces. Use the following commands to clone the repository and navigate into its directory.

```sh
# Clone this repository, copy the URL from the Code button above.
git clone <copied-url>
cd <repository-name>
```

Create the `.env` file in the root of the project. You can use the `.env.template` file as a reference. 

```sh
cp .env.template .env
```
- The `COMETBFT_URL` variable must point to a Namada RPC URL, which can be either public or local. For a public RPC URL, refer to the [Namada Ecosystem Repository](https://github.com/Luminara-Hub/namada-ecosystem/tree/main/user-and-dev-tools/mainnet). If running the Namada Node locally, use the preconfigured `http://host.docker.internal:26657`.
- When running locally, ensure that CometBFT allows RPC calls by setting the the configuration in your `config.toml` file.

Build the required Docker containers for the project.
```sh
docker compose build
```

Launch the Namada Indexer.
```sh
# Run the Docker containers in the foreground, displaying all logs and keeping the terminal active until stopped.
docker compose up

# Run the Docker containers in detached mode, starting them in the background without showing logs in the terminal.
docker compose up -d
```

## License

This project is licensed under the GNU General Public License v3.0. You can
consult a copy of the license text [here](COPYING).
