class Skillhub < Formula
  desc "Universal skill registry for AI agents"
    homepage "https://github.com/Sup4ikX/skillhub-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Sup4ikX/skillhub-cli/releases/download/v#{version}/skillhub-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/Sup4ikX/skillhub-cli/releases/download/v#{version}/skillhub-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "https://github.com/Sup4ikX/skillhub-cli/releases/download/v#{version}/skillhub-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/Sup4ikX/skillhub-cli/releases/download/v#{version}/skillhub-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "skillhub"
  end

  test do
    system "#{bin}/skillhub", "--help"
  end
end
