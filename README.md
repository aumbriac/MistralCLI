# MistralCLI - Rust CLI for Mistral AI API

**This project has no affiliation with Mistral AI.**

MistralCLI is a simple open source command-line interface (CLI) written in Rust that allows you to easily interact with the Mistral AI API.

## Prerequisites

- **Rust**: You can install Rust by following the instructions on the official website: [https://www.rust-lang.org/learn/get-started](https://www.rust-lang.org/learn/get-started).

## Usage

Before you can interact with the API, you need to set your Mistral API key.

### Setting Your Mistral API Key

1. Obtain a Mistral API key from the Mistral website: [https://mistral.ai](https://mistral.ai)
2. Add your API key to your shell environment. You can do this by adding the following line to your `~/.bashrc` (or `~/.bash_profile` on Mac) file: `export MISTRAL_API_KEY=your-api-key`
3. To apply the changes, either restart your shell session or run the following command: `source ~/.bashrc  # or source ~/.bash_profile on Mac`

## Installation

1. Clone the MistralCLI repository to your local machine using Git: `git clone https://github.com/aumbriac/MistralCLI`
2. Enter the directory: `cd MistralCLI`
3. Test the project locally using Cargo (Rust's package manager): `cargo run What is the meaning of life?`
4. Build the executable: `cargo build`
5. Copy the executable to a directory in your system's PATH to make it easily accessible:
   - **Windows**: You can copy it to a directory listed in your system's `Path` environment variable, such as `C:\Windows\System32`.
   - **Mac**: You can copy it to `/usr/local/bin/` using the following command: `sudo cp target/release/mistral /usr/local/bin/`

### Using MistralCLI

Now that you have the MistralCLI installed on your system, you can use it directly from within your shell environment.

- Example usage: `mistral What is the meaning of life?`

Note: `mistral-tiny` is the default model, but you can use either `mistral-small` or `mistral-medium` by specifying an optional `-m` flag with the model suffix: `mistral -m medium What is the meaning of life?`

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
