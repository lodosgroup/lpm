# LOD Package Manager

This is the main source code repository for Lod Package Manager.

- Please refer to the documentation at [lpm.lodosgroup.org/docs](https://lpm.lodosgroup.org/docs/getting-started/introduction) for more information.

- If you'd like to add/update or propose packages for the lpm repositories, please visit [github.com/lodosgroup/package-builds](https://github.com/lodosgroup/package-builds).

- To report bugs, suggest improvements, or request new features, kindly [open a GitHub issue](https://github.com/lodosgroup/lpm/issues/new).

- For security-related concerns, please [open a private security vulnerability report](https://github.com/lodosgroup/lpm/security/advisories/new). We kindly ask you not to create a public issue on GitHub for security matters.

- To explore the packages available in the official lpm repositories, visit [lpm.lodosgroup.org/explore-packages](https://lpm.lodosgroup.org/explore-packages/).

- For guidance on contributing, read [lpm.lodosgroup.org/docs/contributing/contribution-guidelines/](https://lpm.lodosgroup.org/docs/contributing/contribution-guidelines/) and [lpm.lodosgroup.org/docs/contributing/code-of-conduct/](https://lpm.lodosgroup.org/docs/contributing/code-of-conduct/).

- Get the latest news and updates about lpm [lpm.lodosgroup.org/news](https://lpm.lodosgroup.org/news/).

## Quickstart

### Install with Cargo

To install LPM from a specific branch, run the following command:

```sh
cargo install --git https://github.com/lodosgroup/lpm --branch main
```

Alternatively, you can install it from tags:

```sh
cargo install --git https://github.com/lodosgroup/lpm --branch <tag>
```

To confirm a successful LPM installation, simply execute the `lpm -v` or `lpm --version` command.

### Build LPM from Source

If you prefer building LPM from its source code (usually preferred for development), follow these steps:

1. **Clone the lpm repository from GitHub**:

   ```sh
   git clone https://github.com/lodosgroup/lpm
   ```

2. **Change into the cloned repository directory**:

   ```sh
   cd lpm
   ```

3. **Build the lpm executable**:
    

   ```sh
   cargo build --release # exclude the `--release` flag for debugging
   ```

After the building, you will be able to use the lpm executable under`target/{debug/release}` directory.

### Basic Usage

1. **Migrate LPM database**:

    The first step is to migrate the LPM database. This process initializes the core database files required for LPM to function effectively.

    ```sh
    sudo lpm --update --db
    ```

2. **Add repository**:

    Adding a repository is essential for LPM to access and manage packages. A repository acts as the source of packages for your system. Let's add the `linux-amd64-default` repository as an example.

    ```sh
    # args: <repository-name> <repository-url>
    sudo lpm --repository --add linux-amd64-default linux-amd64-default.lpm.lodosgroup.org
    ```

    Once you've added the repository, LPM will synchronize with the package indexes sourced from the added repository. This indicates that you are all set to install packages.

3. **Install a package**:

    Installing packages using LPM is straightforward. Simply use the following command, replacing <package-name> with the name of the package you want to install.

    ```sh
    # args: <package-name>
    sudo lpm --install lzip
    ```

    To confirm the successful completion of the installation, you can check by running the command `lzip --version`.

4. **Delete the installed package**:

    If you want to delete a package from your system, use the delete command followed by the package name.

    ```sh
    # args: <package-name>
    sudo lpm --delete lzip
    ```

These steps cover the basic operations to quickly start using the LOD Package Manager. You can explore the advanced features of LPM from commands section on this website.
