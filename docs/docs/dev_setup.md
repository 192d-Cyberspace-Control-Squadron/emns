# Development enviroment setup

## 1. Install uv  

- Download and install the latest version of Python from [python.org](https://python.org/)
- Install the lastest version of [uv](https://docs.astral.sh/uv/getting-started/installation/)
- Verify the installation by running:
  
```sh
python3 --version
```

## Making Changes

Now you're ready to [clone the repository](https://help.github.com/articles/cloning-a-repository/) locally and start making changes. The Python Applications source code is in the `src`.

Once the repository is cloned install the dependencies.

```sh
uv sync
```

## Submitting an Issue

You should feel free to [submit an issue](https://github.com/192d-Cyberspace-Control-Squadron/borg-agent/issues) on our GitHub repository for anything you find that needs attention on the website. That includes content, functionality, design, or anything else!

### Submitting a Bug Report

When submitting a bug report on the website, please be sure to include accurate and thorough information about the problem you're observing. Be sure to include:

- Steps to reproduce the problem,
- What you expected to happen,
- What actually happend (or didn't happen), and
- Technical details including your Operating System name and version and Web browser name and version number.

## Submitting Code

When making your changes, it is highly encouraged that you use a [branch in Git](https://git-scm.com/book/en/v2/Git-Branching-Basic-Branching-and-Merging), then submit a [pull request](https://github.com/192d-Cyberspace-Control-Squadron/borg-agent/pulls) (PR) on GitHub. Your pull request will go through some automated checks using [Github Actions](https://github.com/features/actions), a continuous integration and deployment tool.

After review by the 192d Cyberspace Control Squadron, your PR will either be commented on with a request for more information or changes, or it will be merged into the `main` branch.