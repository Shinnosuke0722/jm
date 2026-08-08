# Homebrew formula for jm — JDK version manager
# Release template only; no official Homebrew tap is currently published.
#
# Maintainers: after a release, update `version`, `url`, and `sha256`
# for each platform block. The release workflow can automate this via:
#   sed -i "s/VERSION_PLACEHOLDER/x.y.z/g" jm.rb

class Jm < Formula
  desc "Cross-platform JDK and Java version manager"
  homepage "https://github.com/Shinnosuke0722/jm"
  license any_of: ["MIT", "Apache-2.0"]
  version "VERSION_PLACEHOLDER"

  on_macos do
    url "https://github.com/Shinnosuke0722/jm/releases/download/v#{version}/jm-macos-universal.tar.gz"
    sha256 "SHA256_MACOS_PLACEHOLDER"
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
    EOS
  end

  test do
    assert_match "jm #{version}", shell_output("#{bin}/jm --version")
    assert_match "JDK version manager", shell_output("#{bin}/jm --help")
  end
end
