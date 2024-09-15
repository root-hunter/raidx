node-dev:
	cd raidx; cargo run -- RUST_LOG=debug

migrate:
	diesel migration redo
	diesel migration run

start-raid1:
	cd raidx; cargo run -- deamon --configs /home/roothunter/Dev/raidx/config/raid1/raidx.config.json start

start-raid2:
	cd raidx; cargo run -- deamon --configs /home/roothunter/Dev/raidx/config/raid2/raidx.config.json start