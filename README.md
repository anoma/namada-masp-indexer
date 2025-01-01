# 🟡 Namada MASP Indexer

## Status

- 🔧 - This project is a work in progress. 
- 🚧 - Functionality is not guaranteed at this stage. 
- ⚠️ - Use at your own risk.

##  About 

This repository, **Namada MASP Indexer**, is distinct from and incomparable to the similarly named [Namada Indexer](https://github.com/anoma/namada-indexer).

Note that: `Namada Indexer != Namada MASP Indexer && Namada MASP Indexer != Namada Indexer`.

The **Namada MASP Indexer** is a specialized tool designed to crawl the [Namada](https://github.com/anoma/namada) network and extract [MASP](https://github.com/anoma/masp) transaction data. Alongside indexing these transactions, the indexer constructs various data structures to track the state of the current MASP commitment tree, note positions, and more. By exposing this data through an HTTP RPC API, Namada clients can efficiently synchronize with the latest MASP state, reducing the need for frequent remote procedure calls to Namada Nodes.

# 🚀 Getting Started

Set up the project locally by following the steps below, which will help you get a local copy up and running.

For security reasons, it is highly recommended to update the default username and password for your PostgreSQL database. Make sure to modify these credentials in both the `.env` file and the `docker-compose.yml` file.

## Architecture

The Namada MASP Indexer consists of a set of containers, for indexing MASP data and storing it in a PostgreSQL database.

### Microservices & Containers

- `namada-masp-block-index`
- `namada-masp-indexer-crawler`
- `namada-masp-webserver`
- `postgres:16-alpine`

## 🐳 Installation with Docker 

### Prerequisites

Before starting, ensure you have the necessary tools and dependencies installed. Below are the steps to set up the required environment.

- **Packages**: Install prerequisite packages from the APT repository.

```sh
apt-get install -y curl apt-transport-https ca-certificates software-properties-common git nano build-essential
```

- **Docker**: Follow the official instructions provided by Docker to install it: [Install Docker Engine](https://docs.docker.com/engine/install/).

### Usage
Ensure you have the latest repository cloned to maintain compatibility with other Namada interfaces. Use the following commands to clone the repository and navigate into its directory.

```sh
# Clone this repository, copy the URL from the Code button above and use.
git clone <copied-url>
cd <repository-name>
```

Create the `.env` file in the root of the project. You can use the `.env.sample` file as a reference. 

```sh
cp .env.sample .env
```
- The `COMETBFT_URL` variable must point to a Namada RPC URL, which can be either public or local. For a public RPC URL, refer to the [Namada Ecosystem Repository](https://github.com/Luminara-Hub/namada-ecosystem/tree/main/user-and-dev-tools/mainnet). If running the Namada Node locally, use the preconfigured `http://host.docker.internal:26657`. 
- When running locally, ensure that CometBFT allows RPC calls by setting the the configuration in your `config.toml` file.

Build the required Docker containers for the project.
```sh
docker compose build
```

Launch the Namada MASP Indexer Docker containers.
```sh
# Run the Docker containers in the foreground, displaying all logs and keeping the terminal active until stopped.
docker compose up

# Run the Docker containers in detached mode, starting them in the background without showing logs in the terminal.
docker compose up -d
```

## REST API

The API endpoints are described in the `swagger.yml` file located in the project root.

# 📃 License

This project is licensed under the GNU General Public License v3.0. The complete license text is available in the [COPYING](COPYING.md) file.
