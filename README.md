# Greed - A price watcher

<hr>

Greed is a powerful price-watching tool that allows you to monitor changes on
websites and receive timely notifications whenever anything changes. Whether
you are tracking price updates, stock availability, or any other dynamic
content on a website, Greed has got you covered.

## Usage

```sh
docker run --rm -d--name greed -v config.toml:config.toml ghcr.io/chronophylos/greed:main
```

## Configuration

TODO

# README.md for Greed - A Price Watcher

![Greed Logo](https://example.com/path/to/logo.png)

Greed is a powerful price-watching tool that allows you to monitor changes on
websites and receive timely notifications whenever anything changes. Whether
you are tracking price updates, stock availability, or any other dynamic
content on a website, Greed has got you covered.

## Features

- **Easy to Use**: Greed provides a simple and straightforward setup process,
  making it accessible for both beginners and advanced users.

- **Real-Time Notifications**: Stay informed about website changes instantly
  through various notification channels such as email, Slack, or SMS.

- **Customizable Configuration**: Greed allows you to configure the websites
  you want to monitor and the specific elements you are interested in, tailored
  to your needs.

- **Docker Support**: Greed is containerized, making it easy to deploy and run
  in any environment that supports Docker.

## Usage

To get started with Greed, follow these steps:

1. **Install Docker**: If you haven't already, make sure you have Docker
   installed on your system.

2. **Pull the Docker Image**: Run the following command to pull the latest
   Greed Docker image:

   ```sh
   docker pull chronophylos/greed:latest
   ```

3. **Create Configuration**: Create a `greed.toml` file to specify the websites
   you want to monitor and the notification settings. Refer to the
   [Configuration](#configuration) section below for more details on how to
   structure the `greed.toml` file or simply copy the example config
   [`greed.sample.toml`](greed.sample.toml)

4. **Run Greed**: Start Greed by running the Docker container with the mounted
   `greed.toml` file:

   ```sh
   docker run --rm -d --name greed -v /path/to/config.toml:/greed.toml chronophylos/greed:latest
   ```

   Greed will now begin monitoring the websites according to your configuration.

## Configuration

The `config.toml` file is where you define the websites you want to watch and
the corresponding notification settings. Below is an example of how you can
structure the configuration:

```toml
[ntfy]
topic = "some-topic-name"

[[sites]]
name = "Example Site"
url = "https://example.com"
selector = '#someid > div.klass'
notifiers = ["ntfy"]

[[sites.rules]]
type = "OnChange"
```

For a detailed explanation of the configuration options, refer to the
[Configuration
Documentation](https://github.com/chronophylos/greed/wiki/Configuration).

## Contributing

We welcome contributions from the community! If you find a bug, have a feature
request, or want to improve the documentation, please feel free to open an
issue or submit a pull request.

## License

Greed is released under the [MIT License](LICENSE).
