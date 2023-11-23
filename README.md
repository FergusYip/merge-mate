# Merge Mate

This is an extension to [git-branchless](https://github.com/arxanas/git-branchless) that automates the creation and management of stacked pull requests on GitHub.

## Dependencies

- [GitHub CLI](https://github.com/cli/cli)
- [Git Branchless](https://github.com/arxanas/git-branchless)


## Installation

```shell
cargo install --git 'https://github.com/FergusYip/merge-mate' --locked --force
```

## Quick Start

The main and most useful `merge-mate` command is `mm update`. It looks at your current `stack(.)` and will update the pull requests of every branch in the stack.

Suppose you have the following stack and every branch has its own pull request.

```
❯ git sl
◇ 809e18c 3m (main) initial commit
┃
◯ fc6d3ce 2m (feature-foo) add foo
┃
◯ 940199e 2m (feature-bar) add bar
┃
● 08d8ee7 2m (feature-foobar) add foobar
```

Now, let's say that we actually need to refactor some code before introducing `foo`, `bar`, and `foobar`. So, we add a new commit before `add foo`, which results in the following revised stack.

```
❯ git sl
◇ 809e18c 7m (main) initial commit
┃
● ff34308 29s (refactor-code) refactor code
┃
◯ 1d18550 10s (feature-foo) add foo
┃
◯ 6d8214d 10s (feature-bar) add bar
┃
◯ 79506b6 10s (feature-foobar) add foobar
```

We want to merge `refactor-code` first so we force push the entire stack and create a new pull request for it.

```
❯ git submit -c
❯ gh pr create
```

But now we encounter a problem: the diff of the `feature-foo` pull request against `main` includes the `refactor-code` commit. Typically, you would manually update the base branch of the `feature-foo` pull request to be `refactor-code`. And if you have a pull request table of contents, you would have to update it too for every single pull request.


This is where `mm update` comes in. Simply run it to update the base branches and table of contents of every pull request in the stack.

```sh
mm update
```

### Merging Head First

Let's say that we've merged the head of our pull request stack into the main branch.

```
❯ git sl
◇ 809e18c 25m initial commit
┣━┓
⋮ ● 6e45530 44s (refactor-code) refactor code
⋮ ┃
⋮ ◯ b355aaa 44s (feature-foo) add foo
⋮ ┃
⋮ ◯ 6068fda 44s (feature-bar) add bar
⋮ ┃
⋮ ◯ 9ac0be3 44s (feature-foobar) add foobar
⋮
◇ 71f5662 1m (main) Merge branch 'refactor-code' into 'main'
```

The advancement of `main` has caused the diff of the `feature-foo` pull request to become incorrect.

In this scenario, we must rebase `feature-foo` and its descendant on top of `main`.

```sh
git move -s b355aaa -d main
```

Which results in

```
❯ git sl
◇ 809e18c 28m initial commit
┣━┓
⋮ ◯ 6e45530 3m (refactor-code) refactor code
⋮
◇ 71f5662 4m (main) Merge branch 'refactor-code' into 'main'
┃
● c56d427 16s (feature-foo) add foo
┃
◯ 87c9f38 16s (feature-bar) add bar
┃
◯ 0f1e2c9 16s (feature-foobar) add foobar
```

Then we need to push our local changes to GitHub. Make sure you have checked out one of the commits in the stack.

```sh
git submit && mm update
```

We chain `mm update` behind `git submit` to ensure that the head of the `feature-foo` pull request is changed from `refactor-code` to `main`.

### Cleaning Up

In some instances, merged pull requests will not be deleted if they are squashed into `main`.

```
❯ git sl
◇ 809e18c 28m initial commit
┣━┓
⋮ ◯ 6e45530 3m (refactor-code) refactor code
⋮
◇ 71f5662 4m (main) Merge branch 'refactor-code' into 'main'
```

In this case, you would run the `mm cleanup` command. The `mm cleanup` command will look at every branch that is older than `main` and ask if you want to hide it.

```
❯ mm cleanup
Hide "refactor-code"? [Y/n]
```
