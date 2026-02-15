class Vaultura < Formula
  desc "A secure, fully local, terminal-based password manager"
  homepage "https://github.com/zombocoder/vaultura"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/zombocoder/vaultura/releases/download/v0.1.0/vaultura-v0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "40ef988594e0029a8696dc42caa4e11bf59e0cd361361590743eec007e9f3c25"
    else
      url "https://github.com/zombocoder/vaultura/releases/download/v0.1.0/vaultura-v0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "bf0b30a8812b6a93ae23466021619ca5814fa11965ff6c5fa0d5e1d5a4c116b9"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/zombocoder/vaultura/releases/download/v0.1.0/vaultura-v0.1.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "a568cf4b9765daed4fcbbd051c4067d68be7d72dfdbcae7c28227b0270de6a37"
    else
      url "https://github.com/zombocoder/vaultura/releases/download/v0.1.0/vaultura-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "e66ced3b646ee02657765bbd3426542119ecc3ca52277d94dd4dfc9418c10ad1"
    end
  end

  def install
    bin.install "vaultura"
  end

  test do
    assert_match "vaultura", shell_output("#{bin}/vaultura --version")
  end
end
