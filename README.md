<h1 align="center">
    <a href="https://www.rudder.io" target="blank_">
        <img height="100" alt="Rudder logo" src="https://raw.githubusercontent.com/Normation/rudder/master/logo/rudder-logo-rect-black.svg" />
    </a>
    <br>
    System infrastructure automation<br/>for operational security
</h1>

<p align="center">
    <a href="https://www.rudder.io/">Website</a> •
    <a href="https://docs.rudder.io/">Docs</a> •
    <a href="https://chat.rudder.io">Chat</a> •
    <a href="https://www.youtube.com/user/RudderProject">Videos</a> •
    <a href="https://issues.rudder.io/">Bug tracker</a>
    <br><br>
    <a href="https://twitter.com/rudderio">X (Twitter)</a> •
    <a href="https://bsky.app/profile/rudder.io">Bluesky</a> •
    <a href="https://mamot.fr/@rudderio">Mastodon</a> •
    <a href="https://www.linkedin.com/company/rudderbynormation/">LinkedIn</a>
</p>

<div align="center">
    <a href="https://www.rudder.io" target="blank_">
        <img height="400" alt="Rudder logo" src="logo/v.svg" />
    </a>
</div>

## What is Rudder?

Rudder is a system infrastructure automation platform, dedicated to empowering IT operational teams
to enhance cyber-resilience and foster collaboration in Security Operations (SecOps).

### Features

- **Operational Security Automation:** Automate critical IT operations to fortify infrastructure security:
  - Automated systems inventory
  - Patch management
  - Vulnerability management
  - System hardening and security standards compliance
- **Configuration Management:** Streamline and automate configuration tasks for enhanced reliability, with advanced compliance visualization. Configuration policies can be created seamlessly using a visual editor, or YAML code.
- **Multiplatform Support:** Manage diverse environments, including Cloud, hybrid, and on-premise setups, running various operating system (Linux/Windows/AIX)
- **Scalable and Dynamic:** A scalable and dynamic approach to infrastructure management, including a powerful hirarchical configuration data engine and automated classification of managed systems. Typical deployments manage 100s to 1000s of systems, and a single Rudder server can manage more than 10k systems.

### Components

A Rudder installation is made of:

* A central server, providing the Web interface, the [HTTP API](https://docs.rudder.io/api) and the automation engine. It can be extended with plugins.
* (optional) Relays acting as smart proxies between the server and the agents.
* A light agent installed on every managed system. It runs autonomously and checks the state of the system continuously (practicaly, every 5 minutes by default).

## Get started

### Evaluate Rudder

You can browse a demo version of Rudder (with fake data) or ask for a license to
test it on your own test infrastructure.
The documentation also provides [a guide](https://docs.rudder.io/get-started/current/index.html)
to install a test plaform locally using Vagrant.

<div align="center">
    ➡️ <a href="https://demo.rudder.io">👁️ <b>Open the demo interface</b></a> ⬅️
    <br><br>
    ➡️ <a href="https://www.rudder.io/free-trial/">💯 <b>Get a one-month free trial of a fully-featured Rudder server</b></a> ⬅️
    <br><br>
</div>

### Install Rudder

#### Standard installation

Follow the step-by-step instructions of the documentation to setup Rudder:

<div align="center">
    ➡️ <a href="https://docs.rudder.io/reference/current/installation/index.html">📥 <b>Install Rudder</b></a> ⬅️
</div>

#### Quick installation

For a quick setup on Linux systems, you can use an installer script.
For a Rudder server:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://repository.rudder.io/tools/rudder-setup | sh -s -- setup-server latest
```

For a Rudder agent, replace `SERVER` by you Rudder server's IP or hostname:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://repository.rudder.io/tools/rudder-setup | sh -s -- setup-agent latest SERVER
```

For more options and to review the script before running it:

```bash
wget https://repository.rudder.io/tools/rudder-setup
chmod +x rudder-setup
./rudder-setup -h
```

## FAQ

### Is Rudder open-source? Is it free?

Rudder follows an _open-core_ model:

* Rudder _core_ is available for free and totally open-source (under GPLv3 an Apache 2.0). It includes the main Rudder components, Linux agents, plus several plugins.
* The complete Rudder solution, the enterprise version, is paid and partially open-source. A part of the paid plugins and agents are distributed under closed license.
   It also includes other benefits:
  * Additionnal plugins that add major features
  * Support for older versions on Linux distributions
  * Support for aditionnal architecures (ARM & Power)
  * Support for proprietary operating systems: Windows and AIX
  * Extended support (see FAQ)
  * User support

We are strong believers in the merits of free / open-source software, and have turned to open-core to ensure the perenity of the project
(i.e. pay the developers).
Rudder is developped by a profesional team, 

We strive to open-source as much code as possible, without threatening the project viability.

Contrary to a lot of other tools, we made the decision to keep the the Web interface free of access.

### What are the differences between Rudder Core and the full Rudder solution?

* Rudder _core_ is a versatile automation engine for mainstream Linux systems,
thought to give access to ... to small organizations and indivudals
* Rudder is an multiplatform operational security solution
matching the current needs of ... (based on Rudder _core_).

Main traction to conﬁguration management
enabling small businesses and individuals to
start their journey in cyber-resilience.

There are no separate solution, the free version is identical to the core of the full solution.
This means there the packages publicly available users are the same as the packages provided to
subscription users, with the same QA and security processes. This is not a two-speed model, but
a "two-scope" model.

### Do I need to reinstall everything if upgrading from Rudder _core_ to a subscription?

The full Rudder solution is a direct superset of Rudder _core_, so you don't have to
reinstall anything and your server and agents are compatible with the additional
features which come as plugins.

You may just need to change the repository URL on your systems to benefit from the extended
maintenance in the future.

### What is the governance structure of the Rudder project?

The project is managed by the French (🇫🇷 🇪🇺) company of the same name (previously known as _Normation_).
The community is diverse and comes from all over the world.

### Where can I find support for Rudder?

The best option is through a Rudder subscription which includes profesionnal support. Users of Rudder _core_ can find help on the 
community channels.

### How to join the community?

Join our community on [GitHub Discussions](https://github.com/amousset/rudder/discussions) or our [Gitter/Element chat](https://app.gitter.im/#/room/#rudder:gitter.im) for support, feedback, and collaboration.

### How long are Rudder versions maintained?

Rudder uses a `MAJOR.MINOR.PATCH` versionning scheme:

* Minor (or Major) releases: A new minor or major version of Rudder is typically released approximately every 6 months. These releases introduce new features, enhancements, and improvements.
* Patch releases: For the maintenance of existing versions, a new patch release covering all currently maintained versions is released approximately every month. These releases focus on bug fixes and security updates.

Maintenance policy:

* Users of Rudder _core_ have a 3-month window to upgrade to the newest minor or major version after its release. This ensures that users can take advantage of the latest features and security enhancements promptly.
* With a subscription, major releases are maintained for an extended period of 18 to 24 months after their initial release. Subscribers also benefit from an extended upgrade window of 6 to 9 months between minor versions. This extended timeframe allows for strategic planning and execution of version upgrades.

Get the list of currently maintained versions [in the documentation](https://docs.rudder.io/versions).

### What is Rudder technical stack? In which language(s) is it developed?

This infrastructure automation software is crafted with a deliberate choice of dependancies and programming languages to ensure reliability, performance, and maintainability. The core of the software is built using Scala and Rust, each serving a specific purpose to elevate the overall quality of the system.


In summary, the combination of Scala's reliability and expressiveness with Rust's performance and safety features creates a powerful foundation for Rudder. This technology stack ensures that the software is not only robust and secure but also aligns with contemporary development practices.



Our current stack of choice is:

* Scala for the main application backend.
* Rust for system components (network daemons and CLIs). The communication between the nodes and the server is handled with the tokio.
* Elm for frontend. Stable, reliable, secured
* PostgreSQL

Learn more [in the documentation](https://docs.rudder.io/reference/8.0/technical_stack.html).

## Contributing 

### How can I get involved with the Rudder project?

We are open to contributions, and development is made in the open.

All kinds of contributions are welcome, including code, documentation, examples of Rudder use cases,
feedback about your usage, etc.

### Contributor License Agreement

We need a signed individual or entity CLA (Contributor License Agreement) before we can merge any code or documentation to Rudder.
We decided to ask for a CLA to:

* Allow us to enforce the license: in general, only the copyright holder or someone having assignment of the copyright can enforce the license of a program.
* Give us options for the future. For example, this allowed us to relicense Rudder from AGLv3 to GPLv3 to facilitate its adoption.

Given the recent developments in the legal framework surrounding patents, copyrights and their use, we want to be very clear about what we give and wait in return before we can accept a contribution. We want to be able to evolve with all these legal issues, and be able to defend the project if something unpleasant happens, or simply if a contributor changes their mind. Prevention is better than cure!

* If you are making a personal contribution, here is the reference text to consult: [individual CLA](https://sign.signable.app/widget/xs2adbWSXS).
* If you are contributing on behalf of a company, consult this version: [entity CLA](https://sign.signable.app/#/widget/4YpYMVZKWG).

In summary (but you should really read the full text, because it alone has legal value), you state that:

* your contribution is voluntary,
* your work is your original creation,
* you grant a copyright license for your contributions to Normation, the software publisher that develops Rudder in the legal and administrative sense,
* you grant a patent license for your contributions to Normation,
* you are not required to provide support for your contributions.

Our text is based on the CLA provided by the [Harmony Agreement Project](https://www.harmonyagreements.org/). The Harmony agreements are a community group focused on contribution agreements for free and open source software (FOSS).

## Security

Please refer to [Rudder's security process](SECURITY.md).

## License

This project is licensed under GPLv3 license, see the provided [LICENSE](https://github.com/Normation/rudder/blob/master/LICENSE) (or
its [source](http://www.gnu.org/licenses/gpl-3.0.txt)).

We add an exception to the main GPLv3 license that allows to build and use plugins
on top of Rudder with any license, open source or closed/proprietary, see the [LICENSE_EXCEPTION](https://github.com/Normation/rudder/blob/master/LICENSE_EXCEPTION).
