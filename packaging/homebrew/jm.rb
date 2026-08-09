# Homebrew formula template for jm — JDK version manager.
# The release workflow replaces the version and checksum placeholders, then the
# generated formula is published at Shinnosuke0722/homebrew-tap.

class Jm < Formula
  desc "Cross-platform JDK and Java version manager"
  homepage "https://github.com/Shinnosuke0722/jm"
  version "VERSION_PLACEHOLDER"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    on_intel do
      url "https://github.com/Shinnosuke0722/jm/releases/download/v#{version}/jm-macos-universal.tar.gz"
      sha256 "SHA256_MACOS_PLACEHOLDER"
    end
    on_arm do
      url "https://github.com/Shinnosuke0722/jm/releases/download/v#{version}/jm-macos-universal.tar.gz"
      sha256 "SHA256_MACOS_PLACEHOLDER"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Shinnosuke0722/jm/releases/download/v#{version}/jm-linux-x86_64.tar.gz"
      sha256 "SHA256_LINUX_X86_64_PLACEHOLDER"
    end
    on_arm do
      url "https://github.com/Shinnosuke0722/jm/releases/download/v#{version}/jm-linux-aarch64.tar.gz"
      sha256 "SHA256_LINUX_AARCH64_PLACEHOLDER"
    end
  end

  def install
    bin.install "jm"
  end

  def caveats
    <<~EOS
      To enable shell integration, add to your shell config:

        # Bash (~/.bashrc)
        eval "$(jm shell init bash)"

        # Zsh (~/.zshrc)
        eval "$(jm shell init zsh)"

        # Fish (~/.config/fish/config.fish)
        jm shell init fish | source

      Update this Homebrew-managed installation with:

        brew upgrade Shinnosuke0722/tap/jm

      Do not run `jm upgrade` for a Homebrew-managed installation.
    EOS
  end

  test do
    assert_match "jm #{version}", shell_output("#{bin}/jm --version")
    assert_match "JDK version manager", shell_output("#{bin}/jm --help")
  end
end
