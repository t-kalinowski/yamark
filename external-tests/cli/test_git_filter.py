"""Run via `uv run external-tests/run.py --suite cli/test_git_filter.py`."""

from __future__ import annotations

import os
import shlex
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory

import pytest

from _support import decode_output, run_cli_case


def test_git_filter_clean_writes_sentence_per_line_markdown_to_stdout() -> None:
    run_cli_case(
        "yamark git-filter clean --stdin-filename post.md",
        stdin="First sentence. Second sentence? Third sentence!\n",
        stdout="First sentence.\nSecond sentence?\nThird sentence!\n",
        stderr="",
    )


@pytest.mark.parametrize("filter_command", ["clean", "smudge"])
def test_git_filter_compacts_pipe_tables_for_minimal_diffs(
    filter_command: str,
) -> None:
    run_cli_case(
        f"yamark git-filter {filter_command} --stdin-filename post.md",
        stdin=(
            "| package | label |\n"
            "|---|---|\n"
            "| dplyr | tidy |\n"
            "| data | wide |\n"
        ),
        stdout=(
            "| package | label |\n"
            "| --- | --- |\n"
            "| dplyr | tidy |\n"
            "| data | wide |\n"
        ),
        stderr="",
    )


def test_git_filter_help_explains_git_config_and_attributes() -> None:
    result = run_cli_case(
        "yamark git-filter --help",
        stderr="",
        stdout_contains=[
            "Git clean/smudge filter helpers for Markdown files",
            "yamark git-filter adopt",
            "yamark git-filter join",
            "yamark git-filter check",
            "yamark git-filter setup",
            (
                "git config filter.yamark-md.clean "
                '"yamark git-filter clean --stdin-filename %f"'
            ),
            (
                "git config filter.yamark-md.smudge "
                '"yamark git-filter smudge --stdin-filename %f '
                '--markdown-wrap-at-column 72"'
            ),
            "Git only runs the filter for paths matched by attributes",
            "*.md filter=yamark-md",
            "*.qmd filter=yamark-md",
            "*.Rmd filter=yamark-md",
            "*.rmd filter=yamark-md",
        ],
    )

    assert "yamark format --stdin-file-path" not in decode_output(result.stdout)


def test_git_filter_smudge_wraps_markdown_to_configured_column() -> None:
    run_cli_case(
        (
            "yamark git-filter smudge --stdin-filename post.md "
            "--markdown-wrap-at-column 32"
        ),
        stdin="Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda.\n",
        stdout="Alpha beta gamma delta epsilon\nzeta eta theta iota kappa\nlambda.\n",
        stderr="",
    )


def test_git_filter_smudge_wraps_markdown_to_default_column_72() -> None:
    run_cli_case(
        "yamark git-filter smudge --stdin-filename post.md",
        stdin=(
            "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda "
            "mu nu xi omicron pi rho sigma tau.\n"
        ),
        stdout=(
            "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi\n"
            "omicron pi rho sigma tau.\n"
        ),
        stderr="",
    )


def test_git_filter_wrap_overrides_document_wrap_and_preserves_mode_key() -> None:
    run_cli_case(
        (
            "yamark git-filter smudge --stdin-filename post.md "
            "--markdown-wrap-at-column 48"
        ),
        stdin=(
            "---\n"
            "editor_options:\n"
            "  markdown:\n"
            "    mode: gfm\n"
            "    wrap: sentence\n"
            "---\n\n"
            "First sentence. Second sentence? Third sentence!\n"
        ),
        stdout=(
            "---\n"
            "editor_options:\n"
            "  markdown:\n"
            "    mode: gfm\n"
            "    wrap: sentence\n"
            "---\n\n"
            "First sentence. Second sentence? Third sentence!\n"
        ),
        stderr="",
    )


def test_git_filter_clean_formats_markdown_front_matter() -> None:
    run_cli_case(
        "yamark git-filter clean --stdin-filename post.qmd",
        stdin="---\ntags: [rust,yaml]\n---\n\nFirst sentence. Second sentence?\n",
        stdout="---\ntags: [rust, yaml]\n---\n\nFirst sentence.\nSecond sentence?\n",
        stderr="",
    )


def test_git_filter_smudge_formats_markdown_front_matter() -> None:
    run_cli_case(
        "yamark git-filter smudge --stdin-filename post.Rmd",
        stdin="---\ntags: [r,markdown]\n---\n\nBody\n",
        stdout="---\ntags: [r, markdown]\n---\n\nBody\n",
        stderr="",
    )


def test_git_filter_rejects_non_markdown_stdin_filename() -> None:
    run_cli_case(
        "yamark git-filter clean --stdin-filename notes.txt",
        stdin="First sentence. Second sentence?\n",
        status=1,
        stdout="",
        stderr=None,
        stderr_contains="unsupported Git filter path",
    )


def test_git_filter_adopt_stages_shared_attributes_and_normalized_blobs() -> None:
    with TemporaryDirectory(prefix="yamark-pytest-git-") as temp:
        repo = Path(temp)
        path = "docs/post.md"

        init_repo(repo)
        target = repo / path
        target.parent.mkdir(parents=True)
        target.write_text(
            (
                "Alpha beta gamma delta epsilon zeta eta theta iota kappa "
                "lambda mu nu xi omicron pi rho sigma tau. Second sentence?\n"
            ),
            encoding="utf-8",
        )
        run_git(repo, "add", path)
        run_git(repo, "commit", "-m", "Initial docs")

        result = run_yamark(
            repo,
            "git-filter",
            "adopt",
            "--markdown-wrap-at-column",
            "32",
        )

        assert "Adopted yamark Git filter" in result.stdout
        assert (repo / ".gitattributes").read_text(encoding="utf-8") == (
            "*.md filter=yamark-md\n"
            "*.qmd filter=yamark-md\n"
            "*.Rmd filter=yamark-md\n"
            "*.rmd filter=yamark-md\n"
        )
        assert run_git(repo, "diff", "--name-only").stdout == ""
        assert run_git(repo, "diff", "--cached", "--name-only").stdout == (
            ".gitattributes\n"
            f"{path}\n"
        )
        assert run_git(repo, "show", f":{path}").stdout == (
            "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda "
            "mu nu xi omicron pi rho sigma tau.\n"
            "Second sentence?\n"
        )
        assert target.read_text(encoding="utf-8") == (
            "Alpha beta gamma delta epsilon\n"
            "zeta eta theta iota kappa lambda\n"
            "mu nu xi omicron pi rho sigma\n"
            "tau. Second sentence?\n"
        )


def test_git_filter_join_refuses_when_branch_is_behind_upstream() -> None:
    with TemporaryDirectory(prefix="yamark-pytest-git-") as temp:
        root = Path(temp)
        origin = create_origin_with_initial_docs(root)
        contributor = root / "contributor"
        maintainer = root / "maintainer"
        path = "docs/post.md"

        run_git(root, "clone", os.fspath(origin), os.fspath(contributor))
        run_git(root, "clone", os.fspath(origin), os.fspath(maintainer))
        configure_git_user(maintainer)
        run_yamark(maintainer, "git-filter", "adopt", "--markdown-wrap-at-column", "32")
        run_git(maintainer, "commit", "-m", "Adopt yamark filter")
        run_git(maintainer, "push")
        run_git(contributor, "fetch")

        result = run_yamark_failure(contributor, "git-filter", "join")

        assert result.returncode == 1
        assert "Run git pull --ff-only first" in result.stderr
        assert (contributor / path).read_text(encoding="utf-8") == (
            "Alpha beta gamma delta epsilon zeta eta theta iota kappa "
            "lambda mu nu xi omicron pi rho sigma tau. Second sentence?\n"
        )


def test_git_filter_join_installs_local_config_and_keeps_checkout_clean() -> None:
    with TemporaryDirectory(prefix="yamark-pytest-git-") as temp:
        root = Path(temp)
        origin = create_origin_with_initial_docs(root)
        maintainer = root / "maintainer"
        contributor = root / "contributor"
        path = "docs/post.md"

        run_git(root, "clone", os.fspath(origin), os.fspath(maintainer))
        configure_git_user(maintainer)
        run_yamark(maintainer, "git-filter", "adopt", "--markdown-wrap-at-column", "32")
        run_git(maintainer, "commit", "-m", "Adopt yamark filter")
        run_git(maintainer, "push")
        run_git(root, "clone", os.fspath(origin), os.fspath(contributor))

        result = run_yamark(
            contributor,
            "git-filter",
            "join",
            "--markdown-wrap-at-column",
            "32",
        )

        assert "Joined yamark Git filter" in result.stdout
        assert run_git(contributor, "status", "--porcelain").stdout == ""
        info_attributes = contributor / ".git/info/attributes"
        assert not info_attributes.exists() or info_attributes.read_text(
            encoding="utf-8"
        ) == ""
        assert (
            "git-filter clean --stdin-filename %f"
            in run_git(contributor, "config", "filter.yamark-md.clean").stdout
        )
        check = run_yamark(
            contributor,
            "git-filter",
            "check",
            "--markdown-wrap-at-column",
            "32",
        )
        assert "yamark Git filter check passed for 1 Markdown file(s)." in check.stdout
        assert (contributor / path).read_text(encoding="utf-8") == (
            "Alpha beta gamma delta epsilon\n"
            "zeta eta theta iota kappa lambda\n"
            "mu nu xi omicron pi rho sigma\n"
            "tau. Second sentence?\n"
        )


def test_git_filter_check_rejects_non_roundtripping_committed_blob() -> None:
    with TemporaryDirectory(prefix="yamark-pytest-git-") as temp:
        repo = Path(temp)
        path = "docs/post.md"

        init_repo(repo)
        (repo / ".gitattributes").write_text("*.md filter=yamark-md\n", encoding="utf-8")
        target = repo / path
        target.parent.mkdir(parents=True)
        target.write_text("First sentence. Second sentence?\n", encoding="utf-8")
        run_git(repo, "add", ".gitattributes", path)
        run_git(repo, "commit", "-m", "Raw markdown")

        result = run_yamark_failure(repo, "git-filter", "check")

        assert result.returncode == 1
        assert f"{path}: roundtrip failure" in result.stderr
        assert "clean(smudge(blob)) != blob" in result.stderr


def test_git_filter_setup_configures_current_repo_for_specific_path() -> None:
    with TemporaryDirectory(prefix="yamark-pytest-git-") as temp:
        repo = Path(temp)
        path = "content/blog/deep-learning-with-r-3e/index.md"

        run_git(repo, "init")
        target = repo / path
        target.parent.mkdir(parents=True)
        target.write_text("First sentence. Second sentence?\n", encoding="utf-8")

        setup = run_yamark(repo, "git-filter", "setup", path)
        run_yamark(repo, "git-filter", "setup", path)

        for term in [
            "Configured yamark Git filter",
            "git add --renormalize .",
            "yamark git-filter teardown",
            "git -c filter.yamark-md.clean=cat add <path>",
            "NEWS.md -filter",
            "yamark format --wrap 72 NEWS.md",
        ]:
            assert term in setup.stdout

        attributes = (repo / ".git/info/attributes").read_text(encoding="utf-8")
        assert attributes == f"{path} filter=yamark-md\n"
        assert run_git(repo, "config", "filter.yamark-md.required").stdout == "true\n"
        assert run_git(repo, "config", "merge.renormalize").stdout == "true\n"
        assert (
            "git-filter clean --stdin-filename %f"
            in run_git(repo, "config", "filter.yamark-md.clean").stdout
        )
        assert (
            "git-filter smudge --stdin-filename %f --markdown-wrap-at-column 72"
            in run_git(repo, "config", "filter.yamark-md.smudge").stdout
        )
        assert (
            run_git(repo, "check-attr", "filter", "--", path).stdout
            == f"{path}: filter: yamark-md\n"
        )

        run_git(repo, "add", path)
        assert run_git(repo, "show", f":{path}").stdout == (
            "First sentence.\nSecond sentence?\n"
        )


def test_git_filter_teardown_removes_local_setup() -> None:
    with TemporaryDirectory(prefix="yamark-pytest-git-") as temp:
        repo = Path(temp)
        path = "content/blog/post/index.md"

        run_git(repo, "init")
        target = repo / path
        target.parent.mkdir(parents=True)
        target.write_text("First sentence. Second sentence?\n", encoding="utf-8")

        run_yamark(repo, "git-filter", "setup", path)
        attributes_path = repo / ".git/info/attributes"
        with attributes_path.open("a", encoding="utf-8") as attributes:
            attributes.write("NEWS.md -filter\n")
            attributes.write("manual.md filter=yamark-md diff=markdown\n")

        result = run_yamark(repo, "git-filter", "teardown")

        assert "Removed yamark Git filter setup" in result.stdout
        assert "Unset 4 local Git config value(s)" in result.stdout
        assert "Removed 1 yamark attribute pattern(s)" in result.stdout
        assert attributes_path.read_text(encoding="utf-8") == (
            "NEWS.md -filter\n"
            "manual.md filter=yamark-md diff=markdown\n"
        )
        assert (
            run_git(repo, "check-attr", "filter", "--", path).stdout
            == f"{path}: filter: unspecified\n"
        )

        for key in [
            "filter.yamark-md.clean",
            "filter.yamark-md.smudge",
            "filter.yamark-md.required",
            "merge.renormalize",
        ]:
            missing = subprocess.run(
                ["git", "config", "--local", "--get", key],
                cwd=repo,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                text=True,
            )
            assert missing.returncode == 1


def test_git_filter_setup_uses_default_markdown_patterns_for_repo_arg() -> None:
    with TemporaryDirectory(prefix="yamark-pytest-git-") as temp:
        cwd = Path(temp)
        repo = cwd / "repo"
        path = "post.rmd"

        repo.mkdir()
        run_git(repo, "init")
        (repo / path).write_text("First sentence. Second sentence?\n", encoding="utf-8")

        run_yamark(cwd, "git-filter", "setup", "--repo", os.fspath(repo))

        assert (repo / ".git/info/attributes").read_text(encoding="utf-8") == (
            "*.md filter=yamark-md\n"
            "*.qmd filter=yamark-md\n"
            "*.Rmd filter=yamark-md\n"
            "*.rmd filter=yamark-md\n"
        )
        assert (
            run_git(repo, "check-attr", "filter", "--", path).stdout
            == f"{path}: filter: yamark-md\n"
        )

        run_git(repo, "add", path)
        assert run_git(repo, "show", f":{path}").stdout == (
            "First sentence.\nSecond sentence?\n"
        )


def test_git_filter_clean_integrates_with_git_index_and_diff() -> None:
    with TemporaryDirectory(prefix="yamark-pytest-git-") as temp:
        repo = Path(temp)
        yamark = shlex.quote(os.environ["YAMARK_BIN"])

        run_git(repo, "init")
        run_git(
            repo,
            "config",
            "filter.yamark-md.clean",
            f"{yamark} git-filter clean --stdin-filename %f",
        )
        run_git(
            repo,
            "config",
            "filter.yamark-md.smudge",
            (
                f"{yamark} git-filter smudge --stdin-filename %f "
                "--markdown-wrap-at-column 72"
            ),
        )
        run_git(repo, "config", "filter.yamark-md.required", "true")
        (repo / ".git/info/attributes").write_text(
            "*.md filter=yamark-md\n",
            encoding="utf-8",
        )

        (repo / "file.md").write_text(
            (
                "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda. "
                "Second sentence stays here.\n"
            ),
            encoding="utf-8",
        )
        run_git(repo, "add", "file.md")

        index = run_git(repo, "show", ":file.md").stdout
        assert index == (
            "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda.\n"
            "Second sentence stays here.\n"
        )

        (repo / "file.md").write_text(
            (
                "Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda. "
                "Second sentence changes here.\n"
            ),
            encoding="utf-8",
        )

        diff = run_git(
            repo,
            "-c",
            "diff.external=",
            "diff",
            "--no-ext-diff",
            "--",
            "file.md",
        ).stdout
        assert "-Second sentence stays here.\n+Second sentence changes here." in diff
        assert (
            "-Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda. Second"
            not in diff
        )


def run_yamark(cwd: Path, *args: str) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [os.environ["YAMARK_BIN"], *args],
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )
    assert result.returncode == 0, (
        f"yamark {shlex.join(args)} failed with exit code {result.returncode}\n"
        f"stdout:\n{result.stdout}\n"
        f"stderr:\n{result.stderr}"
    )
    return result


def run_yamark_failure(cwd: Path, *args: str) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [os.environ["YAMARK_BIN"], *args],
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )
    assert result.returncode != 0, (
        f"yamark {shlex.join(args)} unexpectedly succeeded\n"
        f"stdout:\n{result.stdout}\n"
        f"stderr:\n{result.stderr}"
    )
    return result


def run_git(repo: Path, *args: str) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        text=True,
    )
    assert result.returncode == 0, (
        f"git {shlex.join(args)} failed with exit code {result.returncode}\n"
        f"stdout:\n{result.stdout}\n"
        f"stderr:\n{result.stderr}"
    )
    return result


def init_repo(repo: Path) -> None:
    run_git(repo, "init", "-b", "main")
    configure_git_user(repo)


def configure_git_user(repo: Path) -> None:
    run_git(repo, "config", "user.email", "yamark@example.test")
    run_git(repo, "config", "user.name", "Yamark Test")


def create_origin_with_initial_docs(root: Path) -> Path:
    source = root / "source"
    origin = root / "origin.git"
    path = "docs/post.md"

    source.mkdir()
    init_repo(source)
    target = source / path
    target.parent.mkdir(parents=True)
    target.write_text(
        (
            "Alpha beta gamma delta epsilon zeta eta theta iota kappa "
            "lambda mu nu xi omicron pi rho sigma tau. Second sentence?\n"
        ),
        encoding="utf-8",
    )
    run_git(source, "add", path)
    run_git(source, "commit", "-m", "Initial docs")
    run_git(root, "clone", "--bare", os.fspath(source), os.fspath(origin))
    return origin
