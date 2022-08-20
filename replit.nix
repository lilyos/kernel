{ pkgs }: {
	deps = [
		pkgs.qemu
  pkgs.rustup
		pkgs.rustfmt
        pkgs.rust-analyzer
	];
}