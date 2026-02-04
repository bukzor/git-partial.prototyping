class GitCommitStaged < Formula
  desc "Commit staged changes at specific paths only (unlike git commit -- paths)"
  homepage "https://github.com/bukzor/git-partial.prototyping"
  url "https://github.com/bukzor/git-partial.prototyping.git",
      branch: "main"
  version "0.1.0"
  license "Apache-2.0"

  head "https://github.com/bukzor/git-partial.prototyping.git", branch: "main"

  depends_on "openssl@3"
  depends_on "pkg-config" => :build
  # Use rustup instead of rust formula; Homebrew's rust bottles have glibc
  # version mismatches on Linux that cause link failures.
  depends_on "rustup" => :build

  def install
    require "open3"
    rustup = Formula["rustup"].opt_bin/"rustup"

    # HOME => nil bypasses Homebrew's fake HOME so rustup finds its toolchains
    cargo_path, status = Open3.capture2({"HOME" => nil}, rustup.to_s, "which", "cargo")
    cargo_path.chomp!

    odie "rustup could not find cargo; run 'rustup default stable'" unless status.success?

    ENV.prepend_path "PATH", File.dirname(cargo_path)

    cd "git-commit-staged" do
      system "cargo", "build", "--release"
      bin.install "../target/release/git-commit-staged"
      bin.install "../target/release/git-commit-files"
      man1.install "man/git-commit-staged.1"
      man1.install "man/git-commit-files.1"
    end
  end

  test do
    # Test help flag works
    output = shell_output("#{bin}/git-commit-staged -h 2>&1")
    assert_match "Commit staged changes at specific paths only", output

    # Test it requires a git repo
    output = shell_output("#{bin}/git-commit-staged . 2>&1", 1)
    assert_match(/could not find repository|not a git repository/i, output)
  end
end
