# Changelog

## [3.0.0-rc.1](https://github.com/bizzkoot/copilot-tracker/compare/v2.4.8-rc.1...v3.0.0-rc.1) (2026-03-15)


### ⚠ BREAKING CHANGES

* Release workflow no longer triggers automatically on push to main. Releases must now be manually triggered via GitHub Actions UI.
* Electron builds discontinued after v1.5.1

### ci

* remove Electron builds and update release workflow ([c5aacbf](https://github.com/bizzkoot/copilot-tracker/commit/c5aacbf198a7f243add9b2161f53fe2fad00959e))


### Features

* add asset validation script ([1afe178](https://github.com/bizzkoot/copilot-tracker/commit/1afe1785775ca3ea3a137531c2662535064d12e7))
* add backup and restore functionality in settings ([4e5450c](https://github.com/bizzkoot/copilot-tracker/commit/4e5450c4973562d7fe4f0b08366bfc2ac8b2dd60))
* add cargo.lock for reproducible builds ([0afc9fb](https://github.com/bizzkoot/copilot-tracker/commit/0afc9fbb3256ee5a18d213473d89f79758c124b7))
* add customizable tray icon display formats ([6972b39](https://github.com/bizzkoot/copilot-tracker/commit/6972b39a8342c72138337f87849da9327b6228d4))
* add environment variables configuration and dev-only logging ([f1b99c9](https://github.com/bizzkoot/copilot-tracker/commit/f1b99c9fbb42c93df6816cf4e225eb3c732ed858))
* Add model-level usage breakdown with expandable history table ([6f4760b](https://github.com/bizzkoot/copilot-tracker/commit/6f4760b39edeb07a1f3440d1dad1eefc8e3bb789))
* add multiplatform Tauri builds to CI/CD workflow ([069f461](https://github.com/bizzkoot/copilot-tracker/commit/069f461c528e659fc0a46a0f21e89cb0438c9f8c))
* add update notification system with native alerts and tray indicators ([edc2666](https://github.com/bizzkoot/copilot-tracker/commit/edc2666f2d73095188e15db9b52e45029652dce7))
* **auth:** implement auth-time usage data capture via URL hash redirect ([ec981c5](https://github.com/bizzkoot/copilot-tracker/commit/ec981c5432595c62a21c2c9f75ca3fe5f9a55685))
* **auth:** implement working hidden webview for silent usage refresh ([0299bf0](https://github.com/bizzkoot/copilot-tracker/commit/0299bf0a9ab524a378465804c5540869c1a82a45))
* convert release workflow to manual trigger with intelligent version bumping ([a235a0d](https://github.com/bizzkoot/copilot-tracker/commit/a235a0d9a63212e9bf39dd3666173e0f0be42cc0))
* **dashboard:** add model usage totals to Daily Breakdown table footer ([c4ef228](https://github.com/bizzkoot/copilot-tracker/commit/c4ef228ed669c9e42ad536b768c2f60e246d7a2d))
* **docs:** add PRD for Tauri migration of Copilot Tracker ([c153360](https://github.com/bizzkoot/copilot-tracker/commit/c1533603ca5d8857e83b945f52856660c196e025))
* **polling:** add dynamic background polling with lifecycle management ([f07bc53](https://github.com/bizzkoot/copilot-tracker/commit/f07bc53247f7665208a857ae5f8cd7aefd05a9e4))
* **refresh:** update tray refresh to trigger re-authentication ([1260fc9](https://github.com/bizzkoot/copilot-tracker/commit/1260fc957987fb620f30964f627cdeabc1ec1a85))
* **tauri:** add linting step to tauri build process ([552a9bb](https://github.com/bizzkoot/copilot-tracker/commit/552a9bb1f1f49b67a353efec1af39a10969286e4))
* **tauri:** complete feature parity with Electron implementation ([c7cbe61](https://github.com/bizzkoot/copilot-tracker/commit/c7cbe610ee15aa6e88ed982a3bfcc6784472b5c2))
* **tauri:** complete migration implementation and build support ([78b892c](https://github.com/bizzkoot/copilot-tracker/commit/78b892c95323260059d6c7dc3153d28713ee004b))
* **tauri:** migrate authentication and dashboard logic from Electron ([f24e5ac](https://github.com/bizzkoot/copilot-tracker/commit/f24e5ac85d33daabb8426af881c3163de0289400))
* **tray:** add monthly prediction banner and dashboard action - release ([a8b4081](https://github.com/bizzkoot/copilot-tracker/commit/a8b4081db3f96d78250b0d1aa593387bfb996360))
* **tray:** enhance system tray menu with consumption metrics ([6ca57f4](https://github.com/bizzkoot/copilot-tracker/commit/6ca57f4801625ee7c798fab07b64ba0df014a2ad))
* **tray:** implement Electron-style tray icon with progress indicator ([07141ac](https://github.com/bizzkoot/copilot-tracker/commit/07141ac27741d5972e70f62925e6371e2d56bd67))
* **tray:** implement Retina-aware text rendering with improved sharpness ([7a2b30c](https://github.com/bizzkoot/copilot-tracker/commit/7a2b30c015cd66b9ec0b85b9a40e61f15197bfe5))
* **tray:** persist refresh/update timestamps to disk ([9ffd5f6](https://github.com/bizzkoot/copilot-tracker/commit/9ffd5f6ded24208197200377adb1e5c36ea17963))
* **ui/backend:** enhance dashboard, unify prediction logic, and fix theme sync ([e5737c6](https://github.com/bizzkoot/copilot-tracker/commit/e5737c6a9af853087f0dff6b2c2953124cb0e14f))
* **ui/dashboard:** release - enhance dashboard with EMA trend line and compact layout ([94e7d67](https://github.com/bizzkoot/copilot-tracker/commit/94e7d67016db08dc5b071e38323cb33d848749ff))
* **widget:** add floating usage widget with state persistence ([6a7ff9c](https://github.com/bizzkoot/copilot-tracker/commit/6a7ff9cb4323bfd5835e613ab6cb1590cc9ebf44))
* **widget:** add reactive header labels for consumed/remaining modes ([0af3dc0](https://github.com/bizzkoot/copilot-tracker/commit/0af3dc07602dd05cae3cca89cc317305363be037))


### Bug Fixes

* add bundle metadata for Windows Tauri build ([611d318](https://github.com/bizzkoot/copilot-tracker/commit/611d318c74e2a137d3ed4daa01f89db0b5a3f71d))
* add macOS traffic light safe area and update dashboard icon ([22f097a](https://github.com/bizzkoot/copilot-tracker/commit/22f097acc9893d93c2f79634bb5223020fe82b70))
* add robust polling restart with debounce and shutdown protection ([1cc6061](https://github.com/bizzkoot/copilot-tracker/commit/1cc60611e4050ef170ef7a82a84d7b099fed45dd))
* add x-release-please-version marker to Cargo.toml ([b8ed7dd](https://github.com/bizzkoot/copilot-tracker/commit/b8ed7dd4dbcb2404f627a09d0a3a7c78f4ad00b2))
* always use entitlement endpoint for Copilot Business usage data ([ff6b788](https://github.com/bizzkoot/copilot-tracker/commit/ff6b788c99acdcf627acf9f83c27fe7967547a83))
* **app:** harden Tauri permissions and remove Electron tooling ([0ccc8b3](https://github.com/bizzkoot/copilot-tracker/commit/0ccc8b31ef7666ab5fa63b5bf196c6479e349fc7))
* **auth:** add extraction failure event handling ([b9f0013](https://github.com/bizzkoot/copilot-tracker/commit/b9f001362964664802419858e31a9253d67cd97b))
* **auth:** implement custom protocol redirect for safer data extraction ([f3024ad](https://github.com/bizzkoot/copilot-tracker/commit/f3024ad7aafcee89e576951c9db22680ea59be24))
* **auth:** implement single-window extraction with custom protocol redirect and auto-navigation ([3a47db9](https://github.com/bizzkoot/copilot-tracker/commit/3a47db9590721b80a3eb6d19381b8065f97f26ec))
* **auth:** prevent invisible Re-Login window on Windows and stop unauthenticated GitHub polling - release ([b744aac](https://github.com/bizzkoot/copilot-tracker/commit/b744aacae9e276727195f2f75e1b13aab4450118))
* **build:** configure Vite output directory to match Tauri frontendDist ([cca2128](https://github.com/bizzkoot/copilot-tracker/commit/cca2128dd9d2735f801fe63b60894ab2284719cd))
* **build:** include widget entry in renderer build - release ([be93fce](https://github.com/bizzkoot/copilot-tracker/commit/be93fce9c24befb22b3f431313e64568b4de5ce5))
* **build:** move legacy Electron code to temp and align Tauri scripts ([a0f89be](https://github.com/bizzkoot/copilot-tracker/commit/a0f89be130d88b9c6d9323cfeaa0bc25c8902560))
* **ci:** add bootstrap-sha to release-please config ([5ad5285](https://github.com/bizzkoot/copilot-tracker/commit/5ad5285039d23453d7c30606407d3c62e53bc078))
* **ci:** align tauri build targets with artifact uploads ([ac30140](https://github.com/bizzkoot/copilot-tracker/commit/ac3014032579b4ef093c3da8db3b15a6122353b9))
* **ci:** correct build script references in workflows ([9134648](https://github.com/bizzkoot/copilot-tracker/commit/9134648507fa0f7ef7d2450b471e9feace4dc4da))
* **ci:** correct changelog-sections configuration ([737172f](https://github.com/bizzkoot/copilot-tracker/commit/737172f592d2b754217871a1f3d5154a500d8523))
* **ci:** correct release-please tag naming convention ([d5358ae](https://github.com/bizzkoot/copilot-tracker/commit/d5358ae0efd07d47df87c4629de56b868385df33))
* **ci:** correct version extraction regex in release workflow ([3b464de](https://github.com/bizzkoot/copilot-tracker/commit/3b464de3414e5d557cd671cafee52efb2baa8da9))
* **ci:** improve contributor auto-detection in release workflow ([9615f3f](https://github.com/bizzkoot/copilot-tracker/commit/9615f3f35ba95439d3c80d0d2bb61b6e32a0c1c0))
* **ci:** make Cargo.lock update deterministic in release PR workflow ([5fc3169](https://github.com/bizzkoot/copilot-tracker/commit/5fc31695033fd70bb4fbcdfeae635d07370d2baf))
* **ci:** release - correct version verification regex and non-blocking Cargo.lock check ([1f1287e](https://github.com/bizzkoot/copilot-tracker/commit/1f1287e718ed9970cc734142b83dc78f7d633ae8))
* **ci:** repair all-contributors config and non-interactive add ([d44704b](https://github.com/bizzkoot/copilot-tracker/commit/d44704b588dc10a57ca3b7f998502052ce14df8b))
* **ci:** resolve active release PR branch in format job ([15291c4](https://github.com/bizzkoot/copilot-tracker/commit/15291c4641fcba2f18df911930f2a9394364ed2e))
* **ci:** resolve windows clippy regressions in update-check flow ([d8bf4c8](https://github.com/bizzkoot/copilot-tracker/commit/d8bf4c8d3a2bbd39d93a2989f6ba8227d320d4b7))
* **ci:** restore universal builds and add manual upload workflow ([2f92c52](https://github.com/bizzkoot/copilot-tracker/commit/2f92c524a38e00a67d9483ce355fe3c2352a5b7f))
* **ci:** run release PR formatting when pr_number exists ([47ed5db](https://github.com/bizzkoot/copilot-tracker/commit/47ed5db74b04399c357ce4f47b1ee8ccc79b076c))
* **ci:** simplify Tauri build commands to avoid target specification issues ([f2bb18e](https://github.com/bizzkoot/copilot-tracker/commit/f2bb18ebf99d32c81612bbb0eac286c79c1e250f))
* **ci:** skip format-release-pr gracefully when no open release PR exists ([b293edf](https://github.com/bizzkoot/copilot-tracker/commit/b293edfe400e92958dd7d10fbbd3953c885ed1fd))
* **ci:** switch to release-please Manifest Mode ([c733f25](https://github.com/bizzkoot/copilot-tracker/commit/c733f25923df49a767684ec82bc821ef3a541fc4))
* **ci:** update bootstrap-sha to skip breaking change ([d853635](https://github.com/bizzkoot/copilot-tracker/commit/d853635efb6781adca4fd482c0e00b4839eb0584))
* **ci:** update Cargo.lock directly in format-release-pr job ([689ec99](https://github.com/bizzkoot/copilot-tracker/commit/689ec99860636b39e892d774f727389236de4ba0))
* **ci:** update version sync configuration for future builds ([69940ad](https://github.com/bizzkoot/copilot-tracker/commit/69940ad8d02e75ecbfbae3ce6b9ffbe674fd718d))
* **ci:** use PAT for release-please to trigger PR checks automatically ([89fae57](https://github.com/bizzkoot/copilot-tracker/commit/89fae579c8fc5e37afbe86f472e58013af127a10))
* **code-quality:** resolve audit findings from P0 to P3-Low ([10a933f](https://github.com/bizzkoot/copilot-tracker/commit/10a933fd19e6b1cc6f4d70c683e0514d338c63f6))
* configure app icons and resolve production asset loading ([fe84671](https://github.com/bizzkoot/copilot-tracker/commit/fe84671943307c7bfa0e244928bf782505b2d1a5))
* configure canvas dependency with fallback ([66f78ba](https://github.com/bizzkoot/copilot-tracker/commit/66f78baa98c6376815eedba21126776fbc9cac26))
* **contributors:** force GitHub profile links ([ead826b](https://github.com/bizzkoot/copilot-tracker/commit/ead826bfcdd7c3319de20826ab404e3eb6b1f7bb))
* **core:** improve startup and init robustness ([70480d4](https://github.com/bizzkoot/copilot-tracker/commit/70480d41b7956d6080dae5c545430bd846f7a330))
* correct Cargo.toml updater in release-please config ([55de77f](https://github.com/bizzkoot/copilot-tracker/commit/55de77f7024c650be4c2758bca9fcf7473dd03f7))
* **data:** implement camelCase JSON parsing and persistent history storage ([b542216](https://github.com/bizzkoot/copilot-tracker/commit/b5422162cb7b87e6b5df501f862d9cc66a0843bc))
* **dev:** auto-clear stale cargo lock before tauri dev ([3764be6](https://github.com/bizzkoot/copilot-tracker/commit/3764be6e90b3c500d496c9d287970201e3102cc2))
* **dev:** prevent port 5173 conflict in tauri dev mode ([04dc9cb](https://github.com/bizzkoot/copilot-tracker/commit/04dc9cbd8fda4f5e65b76c7336984631a2427028))
* **docs:** correct capitalization in README features list ([f980221](https://github.com/bizzkoot/copilot-tracker/commit/f9802214b5848ced2586c2134f21100d0d18561b))
* **electron:** prevent auth window flash and reload loop on Windows ([891e29a](https://github.com/bizzkoot/copilot-tracker/commit/891e29acef7304d45fc3740c263401c42ed047d0))
* **electron:** resolve Windows auth window and clean code - release ([fe102ab](https://github.com/bizzkoot/copilot-tracker/commit/fe102abef3939ffc179c0a259d3c53628d8dbe18))
* explicitly specify TOML type and jsonpath for Cargo.toml ([33fe842](https://github.com/bizzkoot/copilot-tracker/commit/33fe842fe17c75eb132a35e5ea8d2c2e803389c5))
* **main:** update identifier retrieval from context configuration ([88455e2](https://github.com/bizzkoot/copilot-tracker/commit/88455e23b328a4458df955d4d6f95e779d0fe91e))
* modernize Electron API and fix app exit crashes ([36aeaf6](https://github.com/bizzkoot/copilot-tracker/commit/36aeaf610948fb0629fda0829addd7a9a5c6b977))
* **platform:** resolve Linux window management and improve cross-platform compatibility ([f4102f3](https://github.com/bizzkoot/copilot-tracker/commit/f4102f3adf3f4da121fc284cd8f647f7010c0a30))
* **polling:** restart background timer when settings change from UI ([3108223](https://github.com/bizzkoot/copilot-tracker/commit/31082231be7685cced5e114bb50d2fe47985e1fd)), closes [#55](https://github.com/bizzkoot/copilot-tracker/issues/55)
* preserve decimal usage display across app surfaces ([2d94e2f](https://github.com/bizzkoot/copilot-tracker/commit/2d94e2fdbef146be0e35df9203befca706c2aa61))
* prevent settings.json corruption and sync dashboard/tray timestamps ([25793f0](https://github.com/bizzkoot/copilot-tracker/commit/25793f0b2008699d62e23651591c51dfb4468e53))
* properly hide dock icon on macOS using set_activation_policy ([2d957d5](https://github.com/bizzkoot/copilot-tracker/commit/2d957d57b90a8786bc1ce1a24ca43968803efe10))
* redesign widget with circular gauge and compact layout ([a5b7874](https://github.com/bizzkoot/copilot-tracker/commit/a5b787435a029bd74c5715a5173e581c5396e1ef))
* Refactor StoreManager initialization to occur before Tauri builder ([763d27a](https://github.com/bizzkoot/copilot-tracker/commit/763d27a40f4b86731a2fc3712b681ed91f94aa3e))
* reflect tray icon format settings in dashboard and widget progress circles ([675e8d8](https://github.com/bizzkoot/copilot-tracker/commit/675e8d829384dcc23890ad3a249fb2a5859e99ef))
* **release:** correct release-please configuration for Tauri version bumping ([6ab6731](https://github.com/bizzkoot/copilot-tracker/commit/6ab6731599b2aa0c5b5542d578cd223a995cb8a8))
* **release:** include refactor commits in changelog and version bump ([c820b8e](https://github.com/bizzkoot/copilot-tracker/commit/c820b8ebb308abfebe0fcb5ff8ad40368cecc489))
* **release:** remove scheduled trigger for release workflow and clarify conditions for execution ([1fb1646](https://github.com/bizzkoot/copilot-tracker/commit/1fb1646aa0a35e645a477b90bddc85dbf48e8288))
* **release:** Update contributor retrieval to use previous release tag ([4e0d207](https://github.com/bizzkoot/copilot-tracker/commit/4e0d2075940bed1270639dca4fe128d2f9b317e4))
* **release:** use dynamic last-release-sha to respect exact release-as version ([164b8e4](https://github.com/bizzkoot/copilot-tracker/commit/164b8e45176cb328b130f37604d30fc4fb7d0673))
* remove global keyboard shortcuts and hide app from macOS dock ([ec1c0f7](https://github.com/bizzkoot/copilot-tracker/commit/ec1c0f7f69db662001f62b15956aa80521f5754d))
* remove icon array from tauri.conf.json to allow auto-detection of platform icons ([9f80fe6](https://github.com/bizzkoot/copilot-tracker/commit/9f80fe6db080ea60cefa811998773c8950ea22a0))
* **reset:** fix race condition and ensure auth state properly updates ([ecc93f0](https://github.com/bizzkoot/copilot-tracker/commit/ecc93f00f2072b778119acb1106e419eb87d8ff5))
* **reset:** properly clear all data on Reset and logout frontend ([68fa4a7](https://github.com/bizzkoot/copilot-tracker/commit/68fa4a785bc2da850fad8816797cd14c1a3f4d44))
* **reset:** wire Reset button to actual backend reset function ([886e558](https://github.com/bizzkoot/copilot-tracker/commit/886e558683e841c8ef5dbf7daf21034c92e0609e))
* resolve build errors and add comprehensive PR checks ([de023db](https://github.com/bizzkoot/copilot-tracker/commit/de023db9799cee7d4197c0c1042099427506889b))
* resolve clippy warnings needless_return and needless_late_init ([3a8ab18](https://github.com/bizzkoot/copilot-tracker/commit/3a8ab18452956ca399527ab26a2d00dfa04ca0fb))
* resolve date parsing issue and refactor usage fetching ([f552c74](https://github.com/bizzkoot/copilot-tracker/commit/f552c743bfb5d7a9f4fbd2aca3d5ee65ed61f4c8))
* resolve history parsing and renderer display issues ([067c1f1](https://github.com/bizzkoot/copilot-tracker/commit/067c1f1323b3edf3be7723db853d53b3e496a9aa))
* resolve tray-dashboard sync, dock visibility, and startup data issues ([75262c0](https://github.com/bizzkoot/copilot-tracker/commit/75262c0af74bb5fe3cc34cd0def6b36e8dad586a))
* **runtime:** use tauri::async_runtime::spawn instead of tokio::spawn ([45985a3](https://github.com/bizzkoot/copilot-tracker/commit/45985a32da7f820bf652fec0b35e71b9520fb9a8))
* **settings:** prevent theme race condition on window focus/refresh ([a2f9e07](https://github.com/bizzkoot/copilot-tracker/commit/a2f9e079cb91abf9163b9b5ac524b6aa8f599e30))
* **settings:** redesign About panel with collapsible accordion layout ([56c3ee7](https://github.com/bizzkoot/copilot-tracker/commit/56c3ee700078555dda5a488bdd452b131a24bc4a))
* **settings:** resolve Windows settings file sync error by using a single file handle ([0531f29](https://github.com/bizzkoot/copilot-tracker/commit/0531f2997013f33988a6fc8d4b64e51afb0591b9))
* stabilize update-check flow and clean build diagnostics ([8432176](https://github.com/bizzkoot/copilot-tracker/commit/8432176889f770a851bbb4672562928508eefdd1))
* startup authentication and auto-minimize on launch ([6cc332e](https://github.com/bizzkoot/copilot-tracker/commit/6cc332efe68f8ace86a5e60b16253a536bbac03a))
* switch release-please config to Single Package Mode ([3e9d766](https://github.com/bizzkoot/copilot-tracker/commit/3e9d76642977fe777b49037b586d91c0f507f03b))
* sync settings UI with tray and improve canvas types ([1eff249](https://github.com/bizzkoot/copilot-tracker/commit/1eff24942d7bd3ef31459e3c501b79f7339c2510))
* sync Tauri version to 2.0.0 and add automated version sync workflow ([a6d0a19](https://github.com/bizzkoot/copilot-tracker/commit/a6d0a19950fa8268423528ca1e3c73bc40bbe46d))
* sync version to 1.5.1 and improve cross-platform compatibility ([e9abcfd](https://github.com/bizzkoot/copilot-tracker/commit/e9abcfd508d1ff9eab11345ae98a236af378c5ae))
* **tauri:** authentication flow improvements and clippy cleanup ([bf5387a](https://github.com/bizzkoot/copilot-tracker/commit/bf5387a244f8af8c52155bf4b56399f37177a2f0))
* **tauri:** resolve 504MB binary size issue ([ad9ff57](https://github.com/bizzkoot/copilot-tracker/commit/ad9ff573a27e882949b376193fff51c268f37c9b))
* **tauri:** resolve Windows and Linux build failures ([d4fd9b2](https://github.com/bizzkoot/copilot-tracker/commit/d4fd9b22afa604433137470a9ddac979e953cf00))
* **tray-menu:** Compress tray menu by moving quota/activity/forecast into submenu ([f092c21](https://github.com/bizzkoot/copilot-tracker/commit/f092c21934b1ae19d9c7c0d1b931e55b30b3a06f)), closes [#42](https://github.com/bizzkoot/copilot-tracker/issues/42)
* **tray:** align Windows tray text with system UI theme ([11b38cd](https://github.com/bizzkoot/copilot-tracker/commit/11b38cdbdebb139ce30a49f02b381e73664edbd4))
* **tray:** CRITICAL BUG FIX - tray listener was parsing wrong event type ([4989ed9](https://github.com/bizzkoot/copilot-tracker/commit/4989ed9512adffd6705c9290bcd9bf8341bb9e7e))
* **tray:** enable usage history items to fix greyed-out text ([5cb2c7b](https://github.com/bizzkoot/copilot-tracker/commit/5cb2c7bdc490aa68cc0dfb9eade38894325e51a0))
* **tray:** ensure Open Dashboard navigates to main page ([c78ac8c](https://github.com/bizzkoot/copilot-tracker/commit/c78ac8c6d3a362911e1da36bffc417b431a86fb5))
* **tray:** improve Linux system theme detection reliability ([079ee7e](https://github.com/bizzkoot/copilot-tracker/commit/079ee7e041b1aa9fd9d940394b64e99f2c07a007))
* **tray:** navigation to settings and optimize icon generation ([514699e](https://github.com/bizzkoot/copilot-tracker/commit/514699e596dd84fa8ae19bdac44d9de38796e9f2))
* **tray:** remove progress circle from tray icon ([c43bf01](https://github.com/bizzkoot/copilot-tracker/commit/c43bf0115562401937d5e9a740ef88b7c62df9a4))
* **tray:** rename 'target' to 'budget' for clarity ([81956e3](https://github.com/bizzkoot/copilot-tracker/commit/81956e32a0ca4b8d38d10b43989241a9d9462da8))
* **tray:** synchronize tray icon with dashboard in real-time ([a919fca](https://github.com/bizzkoot/copilot-tracker/commit/a919fca1ad3423e1f1fe0b09244d44ed79af3a00))
* **tray:** use macOS-native template tint and system-aware text color ([a35c32f](https://github.com/bizzkoot/copilot-tracker/commit/a35c32fbc5c5d6f192b31fab6a0efe10935c60cc))
* **ui:** improve error message when usage data is unavailable ([72aefd2](https://github.com/bizzkoot/copilot-tracker/commit/72aefd2f8aea44bd4d8a482e6b2b38ced863e26f))
* **ui:** prevent double arrows in tray submenus ([ac26808](https://github.com/bizzkoot/copilot-tracker/commit/ac26808c7be4cadc4e1cc4f73fa5cafd17a424ea))
* **ui:** resolve all 5 dashboard and tray synchronization issues ([899f7bf](https://github.com/bizzkoot/copilot-tracker/commit/899f7bff2d5be13aefc591652acef83d8a4d558b))
* update widget to use Tauri 2.x official API ([0223424](https://github.com/bizzkoot/copilot-tracker/commit/0223424156c63ffbce2cb065c99c1fcfc0571c3d))
* **update-check:** harden Windows checks and stabilize CI lint ([59d702c](https://github.com/bizzkoot/copilot-tracker/commit/59d702c723e943d170ad6d316664206abad0f48a))
* upload release assets using tag_name output instead of github.ref ([4f68eb6](https://github.com/bizzkoot/copilot-tracker/commit/4f68eb6233bedd6dbc2c65ec84408b5a8cff2b65))
* **widget:** add last updated timestamp display ([b816cf6](https://github.com/bizzkoot/copilot-tracker/commit/b816cf63ed5df8f068fbb05acab6ce7b9b5e382d))
* **widget:** clip rounded widget window and mask to remove rectangular outline on transparent windows ([0863db2](https://github.com/bizzkoot/copilot-tracker/commit/0863db2e0a909b65a61c83a5e1431abcfcbe82e7))
* **widget:** ensure widget fills full window height to prevent white bottom bar ([8e453ac](https://github.com/bizzkoot/copilot-tracker/commit/8e453ac21bcea71997dd2caa4ece3f22f69e04e9))
* **widget:** fetch cached usage data on mount to prevent race condition ([54f6f89](https://github.com/bizzkoot/copilot-tracker/commit/54f6f896d139c4cc5b2cdc667784e2f68a912530))
* **widget:** fix position initialization and restore usage data display ([af8e6c0](https://github.com/bizzkoot/copilot-tracker/commit/af8e6c054662b7cd3a73608f3b9b912eedb0ee3c))
* **widget:** harden pin and position persistence ([deaaf71](https://github.com/bizzkoot/copilot-tracker/commit/deaaf71614624373dea41b37b842e8d541ae216d))
* **widget:** harden state persistence paths ([4053925](https://github.com/bizzkoot/copilot-tracker/commit/405392523a9a883906da28ed203378538fcea2c3))
* **widget:** improve dragging, focus handling, and tray menu sync ([20cc42f](https://github.com/bizzkoot/copilot-tracker/commit/20cc42fa287a923dd9380d0c0de5d63dc4f0ca2f))
* **widget:** persist position and sync state between tray and settings ([ec42148](https://github.com/bizzkoot/copilot-tracker/commit/ec42148331b2c1c2506826abfdaca97cf00e107d))
* **widget:** persist widget visibility state across app restarts ([7bb25ff](https://github.com/bizzkoot/copilot-tracker/commit/7bb25ff5c919a4fad2728915d53b3ced0a07ea61))
* **widget:** sync startup tray label with restored visibility ([c8e23c3](https://github.com/bizzkoot/copilot-tracker/commit/c8e23c3b488b87977aa0542c31fb5423d36bb35d))
* **widget:** update positioning API and improve notification handling ([d37444f](https://github.com/bizzkoot/copilot-tracker/commit/d37444f3a5aea4038b2bbe70b5ac7e98fd27b20c))
* **windows:** resolve persistent update check failure on Windows builds ([324198f](https://github.com/bizzkoot/copilot-tracker/commit/324198f5b9c3cef5dde06317a716a671ea8c7207)), closes [#31](https://github.com/bizzkoot/copilot-tracker/issues/31)
* **windows:** resolve update check failure on unsigned builds ([b4dcdd2](https://github.com/bizzkoot/copilot-tracker/commit/b4dcdd282c09085d863d5155bdcabd9413f26bab)), closes [#31](https://github.com/bizzkoot/copilot-tracker/issues/31)


### Refactoring

* **dashboard:** overhaul usage trend chart with 7-day SMA and dynamic pacing ([8e13bc5](https://github.com/bizzkoot/copilot-tracker/commit/8e13bc521b1545e65c6f1f845a8e2190b9c9ad19))
* extract tray icon format constants and add documentation ([ef3a934](https://github.com/bizzkoot/copilot-tracker/commit/ef3a934a14a0b5bf679bccc24e17d77ac6cba670))
* improve code quality and type safety ([a0d8714](https://github.com/bizzkoot/copilot-tracker/commit/a0d871489ef42b25f99a5d2be2966da22acbe2b4))
* **release:** integrate contributor updates into release PR workflow ([4d4175a](https://github.com/bizzkoot/copilot-tracker/commit/4d4175adba6c43b90f90cac00be1fe8e8d0e8ddd))
* resolve code quality issues from audit ([9442e1f](https://github.com/bizzkoot/copilot-tracker/commit/9442e1f4d194344325cc2d56ebed74ac18e65d27))
* **settings:** reorganize settings layout with tabbed interface ([7fec11f](https://github.com/bizzkoot/copilot-tracker/commit/7fec11f3166ee883aa3cb0138934a5fa19978764))
* **tray:** remove unused percentage parameter from tray icon rendering ([9cc75dc](https://github.com/bizzkoot/copilot-tracker/commit/9cc75dc400c408c508bb473dbb96b7257d01d1aa))
* **tray:** simplify menu UI and align daily metrics ([45df14b](https://github.com/bizzkoot/copilot-tracker/commit/45df14bae7dff2260e30d0a32811716bd4e50670))
* **tray:** switch to Roboto Mono Medium font ([49e085b](https://github.com/bizzkoot/copilot-tracker/commit/49e085ba140a4ee8c44bc618e34014c41cda3884))
* **ui:** implement DashboardSkeleton for perceived performance ([2fa896f](https://github.com/bizzkoot/copilot-tracker/commit/2fa896f07587fe19dc85d30099909a33e872a448))


### Other

* bump version to 2.4.8-rc.1 ([62fd942](https://github.com/bizzkoot/copilot-tracker/commit/62fd942599b8c18edc21aed5ffc9873434651a1c))
* ignore worktrees directory ([d49703a](https://github.com/bizzkoot/copilot-tracker/commit/d49703a50227bd280ad442fa529cb75dea283b8a))
* initial commit for copilot-tracker ([a36848f](https://github.com/bizzkoot/copilot-tracker/commit/a36848f141ed6a0324fa259eb6f25b74d5681484))
* **main:** release 1.0.0 ([bef397c](https://github.com/bizzkoot/copilot-tracker/commit/bef397c086728cc9ec191a66bf9e66b8d0ca382d))
* **main:** release 1.0.0 ([d4b0f6b](https://github.com/bizzkoot/copilot-tracker/commit/d4b0f6b903d2d7d6b3ae88a60e98da3dfbc73168))
* **main:** release 1.1.0 ([d32ecba](https://github.com/bizzkoot/copilot-tracker/commit/d32ecba6d8a64c42b86945c66fcbadbe829f7160))
* **main:** release 1.1.0 ([874d88d](https://github.com/bizzkoot/copilot-tracker/commit/874d88d67dac4abf0344755e659669df3ef29873))
* **main:** release 1.1.1 ([7d2a8a6](https://github.com/bizzkoot/copilot-tracker/commit/7d2a8a67fa10d0b94e3acc53c9f5ad1d7f165d4e))
* **main:** release 1.1.1 ([728c306](https://github.com/bizzkoot/copilot-tracker/commit/728c306428cace05815160413e4a5d833cc2922c))
* **main:** release 1.1.2 ([736c28c](https://github.com/bizzkoot/copilot-tracker/commit/736c28c09c2f0b2470e398a87b75dd690786325d))
* **main:** release 1.1.2 ([5e486bf](https://github.com/bizzkoot/copilot-tracker/commit/5e486bf1386333c982fb62757dbb34e16e362b18))
* **main:** release 1.2.0 ([94bf29a](https://github.com/bizzkoot/copilot-tracker/commit/94bf29a592f26928977ec27c2c450e1600d23171))
* **main:** release 1.2.0 ([f03aef5](https://github.com/bizzkoot/copilot-tracker/commit/f03aef586f685b2e1c2bc9d1ac9112606f4c8341))
* **main:** release 1.3.0 ([0af4d84](https://github.com/bizzkoot/copilot-tracker/commit/0af4d84a0ac7bff262603e0de7e000fe383a567b))
* **main:** release 1.3.0 ([42d6b39](https://github.com/bizzkoot/copilot-tracker/commit/42d6b3970257bbd67d0b64d4bbe4f0cdecf1dfe1))
* **main:** release 1.3.1 ([10d50f5](https://github.com/bizzkoot/copilot-tracker/commit/10d50f54b6a679c6e3d29bc3151a33c58197c99c))
* **main:** release 1.3.1 ([664f5f2](https://github.com/bizzkoot/copilot-tracker/commit/664f5f28c7ef9fc937ed99d67b682e3a593bb0e8))
* **main:** release 1.4.0 ([fd0b801](https://github.com/bizzkoot/copilot-tracker/commit/fd0b80161f7134b8ad0883f9034028b89fdaa061))
* **main:** release 1.4.0 ([dee6fdc](https://github.com/bizzkoot/copilot-tracker/commit/dee6fdc69fe54f6c1e56171575310ece645682db))
* **main:** release 1.4.1 ([a875482](https://github.com/bizzkoot/copilot-tracker/commit/a8754822b4f9fa71743629dd695f2466f91f62e2))
* **main:** release 1.4.1 ([bc8319c](https://github.com/bizzkoot/copilot-tracker/commit/bc8319cd6dced148330843e69549039b3adaa155))
* **main:** release 1.4.2 ([420332a](https://github.com/bizzkoot/copilot-tracker/commit/420332a15f4d803d343e4c4df933bdecc1891f06))
* **main:** release 1.4.2 ([c007625](https://github.com/bizzkoot/copilot-tracker/commit/c0076251d284ff7014c8437e87d59846a97bf024))
* **main:** release 1.5.0 ([dfbadbf](https://github.com/bizzkoot/copilot-tracker/commit/dfbadbff46fccee599397bd6c0170768d9c7d939))
* **main:** release 1.5.0 ([ba68a6b](https://github.com/bizzkoot/copilot-tracker/commit/ba68a6bf6a9e06853a5be822a2c11a6ebd1c7c11))
* **main:** release 1.5.1 ([62ec0b3](https://github.com/bizzkoot/copilot-tracker/commit/62ec0b3dae60d6c87f82230484621f8793042949))
* **main:** release 1.5.1 ([00782d5](https://github.com/bizzkoot/copilot-tracker/commit/00782d518a15527028008f7c78b6046f0252d683))
* **main:** release 2.0.0 ([3f10f75](https://github.com/bizzkoot/copilot-tracker/commit/3f10f7503bee503ea1b8f9ef0ff074f374091bb8))
* **main:** release 2.0.0 ([830db32](https://github.com/bizzkoot/copilot-tracker/commit/830db32fb44128f7a2e09048bcffc65d80e97fbb))
* **main:** release 2.0.1 ([944bdf5](https://github.com/bizzkoot/copilot-tracker/commit/944bdf5732de548f31bc36ba42f114c01cb7c038))
* **main:** release 2.0.1 ([8922953](https://github.com/bizzkoot/copilot-tracker/commit/89229530e06bcb382a1463af0d1f216cebb8637e))
* **main:** release 2.1.0 ([8be9ee3](https://github.com/bizzkoot/copilot-tracker/commit/8be9ee3fd2aedbbebff4dfef89b9006f579f5f6d))
* **main:** release 2.1.0 ([72bc1a0](https://github.com/bizzkoot/copilot-tracker/commit/72bc1a0a65aab8d817cac7394c0f7f8a84ea4977))
* **main:** release 2.1.1 ([1c075b3](https://github.com/bizzkoot/copilot-tracker/commit/1c075b37ec33704a1c07c35e996cd8e371ba3736))
* **main:** release 2.1.1 ([5ad9f42](https://github.com/bizzkoot/copilot-tracker/commit/5ad9f4201cddbc341ed3658fcf144a80549cb24b))
* **main:** release 2.1.2 ([2e24b29](https://github.com/bizzkoot/copilot-tracker/commit/2e24b29d3ee2fa93fa64886c4616c69eb250b358))
* **main:** release 2.1.2 ([32db3dd](https://github.com/bizzkoot/copilot-tracker/commit/32db3dd022d4017db538608153f2e88e5561023c))
* **main:** release 2.2.0 ([86f3dfb](https://github.com/bizzkoot/copilot-tracker/commit/86f3dfb35bc8eb02b7ba56c6ddb63bc53c320795))
* **main:** release 2.2.0 ([85323e5](https://github.com/bizzkoot/copilot-tracker/commit/85323e5ef9230d76fd49de3c8e222597e6aa5233))
* **main:** release 2.3.0 ([654aff6](https://github.com/bizzkoot/copilot-tracker/commit/654aff6420d343e1983f1543667a63d1cf3772d9))
* **main:** release 2.3.0 ([91fde0c](https://github.com/bizzkoot/copilot-tracker/commit/91fde0c9c0586f2d72958d43fba5458058cfc745))
* **main:** release 2.3.1 ([ba2ebf0](https://github.com/bizzkoot/copilot-tracker/commit/ba2ebf0d1bc02fb1fee6056493c43baeb14022ca))
* **main:** release 2.3.1 ([f101af1](https://github.com/bizzkoot/copilot-tracker/commit/f101af161679d4136df1266fb45921280be594c7))
* **main:** release 2.3.2 ([8c48527](https://github.com/bizzkoot/copilot-tracker/commit/8c48527d56c60a3534c39254061859fc2f09f409))
* **main:** release 2.3.2 ([ecf8e01](https://github.com/bizzkoot/copilot-tracker/commit/ecf8e0116020253ceb458bedcee77edea03d7144))
* **main:** release 2.3.3 ([901429a](https://github.com/bizzkoot/copilot-tracker/commit/901429ac4770a7ae4ba39c6b5aad85c54eb79131))
* **main:** release 2.3.3 ([d193428](https://github.com/bizzkoot/copilot-tracker/commit/d1934283380219ef680b6648151269a605e9bbec))
* **main:** release 2.3.4 ([c469586](https://github.com/bizzkoot/copilot-tracker/commit/c469586554d1713651044cf5641dad974787007f))
* **main:** release 2.3.4 ([4a0cfc1](https://github.com/bizzkoot/copilot-tracker/commit/4a0cfc13a278c1bea61e432e065c7d72dc6bd2d6))
* **main:** release 2.4.0 ([2f07def](https://github.com/bizzkoot/copilot-tracker/commit/2f07def5adf3f1c23f43073b6e2025ae47b811ec))
* **main:** release 2.4.0 ([6296697](https://github.com/bizzkoot/copilot-tracker/commit/629669727841fe7abe5e5f4eb07e5783499b0389))
* **main:** release 2.4.1 ([74c7f18](https://github.com/bizzkoot/copilot-tracker/commit/74c7f18fc6f156f357c7cc5d66277be361fc4b64))
* **main:** release 2.4.1 ([b2b30b5](https://github.com/bizzkoot/copilot-tracker/commit/b2b30b5e617adda396e83a60360ddb4ed96b2549))
* **main:** release 2.4.2 ([17ebe5b](https://github.com/bizzkoot/copilot-tracker/commit/17ebe5bf42ff40a295d9d53b58567be79157af8d))
* **main:** release 2.4.2 ([040cd56](https://github.com/bizzkoot/copilot-tracker/commit/040cd56d78e84575d066f7559fe454a2b98d41cf))
* **main:** release 2.4.3 ([1f8c55f](https://github.com/bizzkoot/copilot-tracker/commit/1f8c55fb7f6567aa350741b0cf5f9e8b4c239d47))
* **main:** release 2.4.3 ([122345f](https://github.com/bizzkoot/copilot-tracker/commit/122345fbbec2ec354a67772e22c753835530abd7))
* **main:** release 2.4.4 ([c275212](https://github.com/bizzkoot/copilot-tracker/commit/c27521217cd2f2f11a10711fae332b4bab4fe909))
* **main:** release 2.4.4 ([306ed9d](https://github.com/bizzkoot/copilot-tracker/commit/306ed9d31f0a026d09661ae58c70cf5ca6ade720))
* **main:** release 2.4.5 ([f26d0e1](https://github.com/bizzkoot/copilot-tracker/commit/f26d0e16fb1b9334f8fac38995c948cd3a1a31cf))
* **main:** release 2.4.5 ([c39909a](https://github.com/bizzkoot/copilot-tracker/commit/c39909ab2c32a3178b7df1044db9224b9a0552f3))
* **main:** release 2.4.6 ([6e1a4c0](https://github.com/bizzkoot/copilot-tracker/commit/6e1a4c0750ffaaad088ce4076e149426726506f9))
* **main:** release 2.4.6 ([918152f](https://github.com/bizzkoot/copilot-tracker/commit/918152f9b81a83d67209fbf80389db9ebc9d95b1))
* **main:** release 2.4.7 ([6de25e4](https://github.com/bizzkoot/copilot-tracker/commit/6de25e4c817983c0f38006b8c6b3e1998a4db16a))
* **main:** release 2.4.7 ([05bba6d](https://github.com/bizzkoot/copilot-tracker/commit/05bba6d45b58f0f3ae791b32c366688a7f71a5c0))
* project housekeeping, build optimization, and trigger release ([6461a85](https://github.com/bizzkoot/copilot-tracker/commit/6461a851b086ca5723aa4445f4a11bbdbf9f41b7))
* release v1.4.0 ([7889f9f](https://github.com/bizzkoot/copilot-tracker/commit/7889f9f9dd5e4be77742f8f16623212f1be53509))
* sync Tauri version files to v2.0.1 ([4dec9da](https://github.com/bizzkoot/copilot-tracker/commit/4dec9dae2459fff8bf5143977f582a02dc37888b))
* sync Tauri version files to v2.1.0 ([db348c9](https://github.com/bizzkoot/copilot-tracker/commit/db348c931f65625013c3942bb89492b4baa33c1b))
* trigger release-please PR creation ([1ce840f](https://github.com/bizzkoot/copilot-tracker/commit/1ce840f301408b4f946258259bda2b48f91d1490))


### Documentation

* add build instructions to README ([8990f0a](https://github.com/bizzkoot/copilot-tracker/commit/8990f0a572a1ccf5df81267bf09d8168e112fd5c))
* Add comprehensive project planning documents ([acb640e](https://github.com/bizzkoot/copilot-tracker/commit/acb640eb68819f208d233c4019e11f7dd37570ab))
* add CONTRIBUTING.md, All Contributors config, and auto-contributor workflow ([1787ee5](https://github.com/bizzkoot/copilot-tracker/commit/1787ee53916d0c46fa8ca942351755916c3d15db))
* add security warnings for unsigned builds ([c6ccafc](https://github.com/bizzkoot/copilot-tracker/commit/c6ccafccbe29380040a938dbd43ddd72dd69a792))
* comprehensive update to README (privacy, offline, troubleshooting) ([88d7959](https://github.com/bizzkoot/copilot-tracker/commit/88d795957168aafd78c6b0a366f06a7747fad306))
* correct Tauri artifact naming in README ([7a276c9](https://github.com/bizzkoot/copilot-tracker/commit/7a276c998bb9ffb697994b41387dfa9a2f29b36e))
* expand WebView2 runtime flow design ([b6c8786](https://github.com/bizzkoot/copilot-tracker/commit/b6c87860f7b74ce2c30d217f476ab769d3e33d9d))
* **readme:** update features and data location, add new assets for dashboard and taskbar ([37c2f98](https://github.com/bizzkoot/copilot-tracker/commit/37c2f98d3da58084d38b1ad7f6f95f5550b725e2))
* RELEASE_WORKFLOW.md with usage guide ([a235a0d](https://github.com/bizzkoot/copilot-tracker/commit/a235a0d9a63212e9bf39dd3666173e0f0be42cc0))
* simplify macOS installation guide and update demo asset ([bce9e8a](https://github.com/bizzkoot/copilot-tracker/commit/bce9e8a487d27f859b7f51b1476dac0e62cb8fd1))
* update AGENTS.md to enhance guidelines and structure for AI agents ([fa17320](https://github.com/bizzkoot/copilot-tracker/commit/fa17320ec5a7493181518583b19ef76683408d4a))
* update README and assets for version 2.4.5 ([652cd32](https://github.com/bizzkoot/copilot-tracker/commit/652cd32c2ee83857a6c86943ea69e632bf284d44))
* **widget:** add new widget image asset ([01d8385](https://github.com/bizzkoot/copilot-tracker/commit/01d83852bb8ca33f2d0b3d88c43f2b0bb1148309))

## [2.4.7](https://github.com/bizzkoot/copilot-tracker/compare/v2.4.6...v2.4.7) (2026-03-05)

### Bug Fixes

- **ci:** make Cargo.lock update deterministic in release PR workflow ([5fc3169](https://github.com/bizzkoot/copilot-tracker/commit/5fc31695033fd70bb4fbcdfeae635d07370d2baf))
- **ci:** resolve active release PR branch in format job ([15291c4](https://github.com/bizzkoot/copilot-tracker/commit/15291c4641fcba2f18df911930f2a9394364ed2e))
- **ci:** run release PR formatting when pr_number exists ([47ed5db](https://github.com/bizzkoot/copilot-tracker/commit/47ed5db74b04399c357ce4f47b1ee8ccc79b076c))
- **ci:** update Cargo.lock directly in format-release-pr job ([689ec99](https://github.com/bizzkoot/copilot-tracker/commit/689ec99860636b39e892d774f727389236de4ba0))
- prevent settings.json corruption and sync dashboard/tray timestamps ([25793f0](https://github.com/bizzkoot/copilot-tracker/commit/25793f0b2008699d62e23651591c51dfb4468e53))

## [2.4.6](https://github.com/bizzkoot/copilot-tracker/compare/v2.4.5...v2.4.6) (2026-02-28)

### Bug Fixes

- preserve decimal usage display across app surfaces ([2d94e2f](https://github.com/bizzkoot/copilot-tracker/commit/2d94e2fdbef146be0e35df9203befca706c2aa61))
- **release:** include refactor commits in changelog and version bump ([c820b8e](https://github.com/bizzkoot/copilot-tracker/commit/c820b8ebb308abfebe0fcb5ff8ad40368cecc489))
- **tray:** enable usage history items to fix greyed-out text ([5cb2c7b](https://github.com/bizzkoot/copilot-tracker/commit/5cb2c7bdc490aa68cc0dfb9eade38894325e51a0))
- **tray:** rename 'target' to 'budget' for clarity ([81956e3](https://github.com/bizzkoot/copilot-tracker/commit/81956e32a0ca4b8d38d10b43989241a9d9462da8))
- **ui:** prevent double arrows in tray submenus ([ac26808](https://github.com/bizzkoot/copilot-tracker/commit/ac26808c7be4cadc4e1cc4f73fa5cafd17a424ea))

### Refactoring

- **dashboard:** overhaul usage trend chart with 7-day SMA and dynamic pacing ([8e13bc5](https://github.com/bizzkoot/copilot-tracker/commit/8e13bc521b1545e65c6f1f845a8e2190b9c9ad19))
- **tray:** simplify menu UI and align daily metrics ([45df14b](https://github.com/bizzkoot/copilot-tracker/commit/45df14bae7dff2260e30d0a32811716bd4e50670))
- **ui:** implement DashboardSkeleton for perceived performance ([2fa896f](https://github.com/bizzkoot/copilot-tracker/commit/2fa896f07587fe19dc85d30099909a33e872a448))

### Documentation

- update README and assets for version 2.4.5 ([652cd32](https://github.com/bizzkoot/copilot-tracker/commit/652cd32c2ee83857a6c86943ea69e632bf284d44))

## [2.4.5](https://github.com/bizzkoot/copilot-tracker/compare/v2.4.4...v2.4.5) (2026-02-23)

### Bug Fixes

- **ci:** improve contributor auto-detection in release workflow ([9615f3f](https://github.com/bizzkoot/copilot-tracker/commit/9615f3f35ba95439d3c80d0d2bb61b6e32a0c1c0))
- **ci:** repair all-contributors config and non-interactive add ([d44704b](https://github.com/bizzkoot/copilot-tracker/commit/d44704b588dc10a57ca3b7f998502052ce14df8b))
- **release:** Update contributor retrieval to use previous release tag ([4e0d207](https://github.com/bizzkoot/copilot-tracker/commit/4e0d2075940bed1270639dca4fe128d2f9b317e4))
- **tray-menu:** Compress tray menu by moving quota/activity/forecast into submenu ([f092c21](https://github.com/bizzkoot/copilot-tracker/commit/f092c21934b1ae19d9c7c0d1b931e55b30b3a06f)), closes [#42](https://github.com/bizzkoot/copilot-tracker/issues/42)

### Documentation

- add CONTRIBUTING.md, All Contributors config, and auto-contributor workflow ([1787ee5](https://github.com/bizzkoot/copilot-tracker/commit/1787ee53916d0c46fa8ca942351755916c3d15db))

## [2.4.4](https://github.com/bizzkoot/copilot-tracker/compare/v2.4.3...v2.4.4) (2026-02-22)

### Bug Fixes

- **widget:** harden pin and position persistence ([deaaf71](https://github.com/bizzkoot/copilot-tracker/commit/deaaf71614624373dea41b37b842e8d541ae216d))
- **widget:** harden state persistence paths ([4053925](https://github.com/bizzkoot/copilot-tracker/commit/405392523a9a883906da28ed203378538fcea2c3))
- **widget:** sync startup tray label with restored visibility ([c8e23c3](https://github.com/bizzkoot/copilot-tracker/commit/c8e23c3b488b87977aa0542c31fb5423d36bb35d))

### Documentation

- update AGENTS.md to enhance guidelines and structure for AI agents ([fa17320](https://github.com/bizzkoot/copilot-tracker/commit/fa17320ec5a7493181518583b19ef76683408d4a))

## [2.4.3](https://github.com/bizzkoot/copilot-tracker/compare/v2.4.2...v2.4.3) (2026-02-19)

### Bug Fixes

- **ci:** align tauri build targets with artifact uploads ([ac30140](https://github.com/bizzkoot/copilot-tracker/commit/ac3014032579b4ef093c3da8db3b15a6122353b9))
- **ci:** resolve windows clippy regressions in update-check flow ([d8bf4c8](https://github.com/bizzkoot/copilot-tracker/commit/d8bf4c8d3a2bbd39d93a2989f6ba8227d320d4b7))
- resolve clippy warnings needless_return and needless_late_init ([3a8ab18](https://github.com/bizzkoot/copilot-tracker/commit/3a8ab18452956ca399527ab26a2d00dfa04ca0fb))
- stabilize update-check flow and clean build diagnostics ([8432176](https://github.com/bizzkoot/copilot-tracker/commit/8432176889f770a851bbb4672562928508eefdd1))
- **update-check:** harden Windows checks and stabilize CI lint ([59d702c](https://github.com/bizzkoot/copilot-tracker/commit/59d702c723e943d170ad6d316664206abad0f48a))
- **windows:** resolve persistent update check failure on Windows builds ([324198f](https://github.com/bizzkoot/copilot-tracker/commit/324198f5b9c3cef5dde06317a716a671ea8c7207)), closes [#31](https://github.com/bizzkoot/copilot-tracker/issues/31)

## [2.4.2](https://github.com/bizzkoot/copilot-tracker/compare/v2.4.1...v2.4.2) (2026-02-19)

### Bug Fixes

- **ci:** restore universal builds and add manual upload workflow ([2f92c52](https://github.com/bizzkoot/copilot-tracker/commit/2f92c524a38e00a67d9483ce355fe3c2352a5b7f))
- **release:** remove scheduled trigger for release workflow and clarify conditions for execution ([1fb1646](https://github.com/bizzkoot/copilot-tracker/commit/1fb1646aa0a35e645a477b90bddc85dbf48e8288))
- **settings:** resolve Windows settings file sync error by using a single file handle ([0531f29](https://github.com/bizzkoot/copilot-tracker/commit/0531f2997013f33988a6fc8d4b64e51afb0591b9))

## [2.4.1](https://github.com/bizzkoot/copilot-tracker/compare/v2.4.0...v2.4.1) (2026-02-18)

### Bug Fixes

- **app:** harden Tauri permissions and remove Electron tooling ([0ccc8b3](https://github.com/bizzkoot/copilot-tracker/commit/0ccc8b31ef7666ab5fa63b5bf196c6479e349fc7))
- **auth:** add extraction failure event handling ([b9f0013](https://github.com/bizzkoot/copilot-tracker/commit/b9f001362964664802419858e31a9253d67cd97b))
- **build:** configure Vite output directory to match Tauri frontendDist ([cca2128](https://github.com/bizzkoot/copilot-tracker/commit/cca2128dd9d2735f801fe63b60894ab2284719cd))
- **build:** move legacy Electron code to temp and align Tauri scripts ([a0f89be](https://github.com/bizzkoot/copilot-tracker/commit/a0f89be130d88b9c6d9323cfeaa0bc25c8902560))
- **ci:** correct build script references in workflows ([9134648](https://github.com/bizzkoot/copilot-tracker/commit/9134648507fa0f7ef7d2450b471e9feace4dc4da))
- **ci:** simplify Tauri build commands to avoid target specification issues ([f2bb18e](https://github.com/bizzkoot/copilot-tracker/commit/f2bb18ebf99d32c81612bbb0eac286c79c1e250f))
- **settings:** redesign About panel with collapsible accordion layout ([56c3ee7](https://github.com/bizzkoot/copilot-tracker/commit/56c3ee700078555dda5a488bdd452b131a24bc4a))
- **widget:** persist widget visibility state across app restarts ([7bb25ff](https://github.com/bizzkoot/copilot-tracker/commit/7bb25ff5c919a4fad2728915d53b3ced0a07ea61))

## [2.4.0](https://github.com/bizzkoot/copilot-tracker/compare/v2.3.4...v2.4.0) (2026-02-14)

### Features

- **tray:** persist refresh/update timestamps to disk ([9ffd5f6](https://github.com/bizzkoot/copilot-tracker/commit/9ffd5f6ded24208197200377adb1e5c36ea17963))
- **widget:** add reactive header labels for consumed/remaining modes ([0af3dc0](https://github.com/bizzkoot/copilot-tracker/commit/0af3dc07602dd05cae3cca89cc317305363be037))

### Bug Fixes

- **dev:** auto-clear stale cargo lock before tauri dev ([3764be6](https://github.com/bizzkoot/copilot-tracker/commit/3764be6e90b3c500d496c9d287970201e3102cc2))
- redesign widget with circular gauge and compact layout ([a5b7874](https://github.com/bizzkoot/copilot-tracker/commit/a5b787435a029bd74c5715a5173e581c5396e1ef))
- reflect tray icon format settings in dashboard and widget progress circles ([675e8d8](https://github.com/bizzkoot/copilot-tracker/commit/675e8d829384dcc23890ad3a249fb2a5859e99ef))
- **tray:** align Windows tray text with system UI theme ([11b38cd](https://github.com/bizzkoot/copilot-tracker/commit/11b38cdbdebb139ce30a49f02b381e73664edbd4))
- **tray:** improve Linux system theme detection reliability ([079ee7e](https://github.com/bizzkoot/copilot-tracker/commit/079ee7e041b1aa9fd9d940394b64e99f2c07a007))
- **tray:** use macOS-native template tint and system-aware text color ([a35c32f](https://github.com/bizzkoot/copilot-tracker/commit/a35c32fbc5c5d6f192b31fab6a0efe10935c60cc))
- **widget:** add last updated timestamp display ([b816cf6](https://github.com/bizzkoot/copilot-tracker/commit/b816cf63ed5df8f068fbb05acab6ce7b9b5e382d))
- **widget:** clip rounded widget window and mask to remove rectangular outline on transparent windows ([0863db2](https://github.com/bizzkoot/copilot-tracker/commit/0863db2e0a909b65a61c83a5e1431abcfcbe82e7))
- **windows:** resolve update check failure on unsigned builds ([b4dcdd2](https://github.com/bizzkoot/copilot-tracker/commit/b4dcdd282c09085d863d5155bdcabd9413f26bab)), closes [#31](https://github.com/bizzkoot/copilot-tracker/issues/31)

### Documentation

- **readme:** update features and data location, add new assets for dashboard and taskbar ([37c2f98](https://github.com/bizzkoot/copilot-tracker/commit/37c2f98d3da58084d38b1ad7f6f95f5550b725e2))

## [2.3.4](https://github.com/bizzkoot/copilot-tracker/compare/v2.3.3...v2.3.4) (2026-02-12)

### Bug Fixes

- **main:** update identifier retrieval from context configuration ([88455e2](https://github.com/bizzkoot/copilot-tracker/commit/88455e23b328a4458df955d4d6f95e779d0fe91e))
- Refactor StoreManager initialization to occur before Tauri builder ([763d27a](https://github.com/bizzkoot/copilot-tracker/commit/763d27a40f4b86731a2fc3712b681ed91f94aa3e))

## [2.3.3](https://github.com/bizzkoot/copilot-tracker/compare/v2.3.2...v2.3.3) (2026-02-11)

### Bug Fixes

- add robust polling restart with debounce and shutdown protection ([1cc6061](https://github.com/bizzkoot/copilot-tracker/commit/1cc60611e4050ef170ef7a82a84d7b099fed45dd))
- update widget to use Tauri 2.x official API ([0223424](https://github.com/bizzkoot/copilot-tracker/commit/0223424156c63ffbce2cb065c99c1fcfc0571c3d))
- **widget:** ensure widget fills full window height to prevent white bottom bar ([8e453ac](https://github.com/bizzkoot/copilot-tracker/commit/8e453ac21bcea71997dd2caa4ece3f22f69e04e9))
- **widget:** fetch cached usage data on mount to prevent race condition ([54f6f89](https://github.com/bizzkoot/copilot-tracker/commit/54f6f896d139c4cc5b2cdc667784e2f68a912530))
- **widget:** fix position initialization and restore usage data display ([af8e6c0](https://github.com/bizzkoot/copilot-tracker/commit/af8e6c054662b7cd3a73608f3b9b912eedb0ee3c))
- **widget:** improve dragging, focus handling, and tray menu sync ([20cc42f](https://github.com/bizzkoot/copilot-tracker/commit/20cc42fa287a923dd9380d0c0de5d63dc4f0ca2f))
- **widget:** persist position and sync state between tray and settings ([ec42148](https://github.com/bizzkoot/copilot-tracker/commit/ec42148331b2c1c2506826abfdaca97cf00e107d))

### Documentation

- **widget:** add new widget image asset ([01d8385](https://github.com/bizzkoot/copilot-tracker/commit/01d83852bb8ca33f2d0b3d88c43f2b0bb1148309))

## [2.3.2](https://github.com/bizzkoot/copilot-tracker/compare/v2.3.1...v2.3.2) (2026-02-10)

### Bug Fixes

- **build:** include widget entry in renderer build - release ([be93fce](https://github.com/bizzkoot/copilot-tracker/commit/be93fce9c24befb22b3f431313e64568b4de5ce5))
- **widget:** update positioning API and improve notification handling ([d37444f](https://github.com/bizzkoot/copilot-tracker/commit/d37444f3a5aea4038b2bbe70b5ac7e98fd27b20c))

## [2.3.1](https://github.com/bizzkoot/copilot-tracker/compare/v2.3.0...v2.3.1) (2026-02-10)

### Bug Fixes

- **ci:** use PAT for release-please to trigger PR checks automatically ([89fae57](https://github.com/bizzkoot/copilot-tracker/commit/89fae579c8fc5e37afbe86f472e58013af127a10))
- **code-quality:** resolve audit findings from P0 to P3-Low ([10a933f](https://github.com/bizzkoot/copilot-tracker/commit/10a933fd19e6b1cc6f4d70c683e0514d338c63f6))
- resolve build errors and add comprehensive PR checks ([de023db](https://github.com/bizzkoot/copilot-tracker/commit/de023db9799cee7d4197c0c1042099427506889b))

### Other

- trigger release-please PR creation ([1ce840f](https://github.com/bizzkoot/copilot-tracker/commit/1ce840f301408b4f946258259bda2b48f91d1490))

## [2.3.0](https://github.com/bizzkoot/copilot-tracker/compare/v2.2.0...v2.3.0) (2026-02-10)

### Features

- **widget:** add floating usage widget with state persistence ([6a7ff9c](https://github.com/bizzkoot/copilot-tracker/commit/6a7ff9cb4323bfd5835e613ab6cb1590cc9ebf44))

### Bug Fixes

- **core:** improve startup and init robustness ([70480d4](https://github.com/bizzkoot/copilot-tracker/commit/70480d41b7956d6080dae5c545430bd846f7a330))
- **tray:** ensure Open Dashboard navigates to main page ([c78ac8c](https://github.com/bizzkoot/copilot-tracker/commit/c78ac8c6d3a362911e1da36bffc417b431a86fb5))

## [2.2.0](https://github.com/bizzkoot/copilot-tracker/compare/v2.1.2...v2.2.0) (2026-02-08)

### Features

- Add model-level usage breakdown with expandable history table ([6f4760b](https://github.com/bizzkoot/copilot-tracker/commit/6f4760b39edeb07a1f3440d1dad1eefc8e3bb789))
- **dashboard:** add model usage totals to Daily Breakdown table footer ([c4ef228](https://github.com/bizzkoot/copilot-tracker/commit/c4ef228ed669c9e42ad536b768c2f60e246d7a2d))
- **ui/backend:** enhance dashboard, unify prediction logic, and fix theme sync ([e5737c6](https://github.com/bizzkoot/copilot-tracker/commit/e5737c6a9af853087f0dff6b2c2953124cb0e14f))
- **ui/dashboard:** release - enhance dashboard with EMA trend line and compact layout ([94e7d67](https://github.com/bizzkoot/copilot-tracker/commit/94e7d67016db08dc5b071e38323cb33d848749ff))

### Bug Fixes

- **dev:** prevent port 5173 conflict in tauri dev mode ([04dc9cb](https://github.com/bizzkoot/copilot-tracker/commit/04dc9cbd8fda4f5e65b76c7336984631a2427028))
- resolve date parsing issue and refactor usage fetching ([f552c74](https://github.com/bizzkoot/copilot-tracker/commit/f552c743bfb5d7a9f4fbd2aca3d5ee65ed61f4c8))
- **settings:** prevent theme race condition on window focus/refresh ([a2f9e07](https://github.com/bizzkoot/copilot-tracker/commit/a2f9e079cb91abf9163b9b5ac524b6aa8f599e30))

## [2.1.2](https://github.com/bizzkoot/copilot-tracker/compare/v2.1.1...v2.1.2) (2026-02-07)

### Bug Fixes

- **ci:** correct version extraction regex in release workflow ([3b464de](https://github.com/bizzkoot/copilot-tracker/commit/3b464de3414e5d557cd671cafee52efb2baa8da9))
- **ci:** release - correct version verification regex and non-blocking Cargo.lock check ([1f1287e](https://github.com/bizzkoot/copilot-tracker/commit/1f1287e718ed9970cc734142b83dc78f7d633ae8))

## [2.1.1](https://github.com/bizzkoot/copilot-tracker/compare/v2.1.0...v2.1.1) (2026-02-07)

### Bug Fixes

- add x-release-please-version marker to Cargo.toml ([b8ed7dd](https://github.com/bizzkoot/copilot-tracker/commit/b8ed7dd4dbcb2404f627a09d0a3a7c78f4ad00b2))
- **ci:** add bootstrap-sha to release-please config ([5ad5285](https://github.com/bizzkoot/copilot-tracker/commit/5ad5285039d23453d7c30606407d3c62e53bc078))
- **ci:** correct changelog-sections configuration ([737172f](https://github.com/bizzkoot/copilot-tracker/commit/737172f592d2b754217871a1f3d5154a500d8523))
- **ci:** correct release-please tag naming convention ([d5358ae](https://github.com/bizzkoot/copilot-tracker/commit/d5358ae0efd07d47df87c4629de56b868385df33))
- **ci:** switch to release-please Manifest Mode ([c733f25](https://github.com/bizzkoot/copilot-tracker/commit/c733f25923df49a767684ec82bc821ef3a541fc4))
- **ci:** update bootstrap-sha to skip breaking change ([d853635](https://github.com/bizzkoot/copilot-tracker/commit/d853635efb6781adca4fd482c0e00b4839eb0584))
- **ci:** update version sync configuration for future builds ([69940ad](https://github.com/bizzkoot/copilot-tracker/commit/69940ad8d02e75ecbfbae3ce6b9ffbe674fd718d))
- correct Cargo.toml updater in release-please config ([55de77f](https://github.com/bizzkoot/copilot-tracker/commit/55de77f7024c650be4c2758bca9fcf7473dd03f7))
- **docs:** correct capitalization in README features list ([f980221](https://github.com/bizzkoot/copilot-tracker/commit/f9802214b5848ced2586c2134f21100d0d18561b))
- switch release-please config to Single Package Mode ([3e9d766](https://github.com/bizzkoot/copilot-tracker/commit/3e9d76642977fe777b49037b586d91c0f507f03b))
- **ui:** improve error message when usage data is unavailable ([72aefd2](https://github.com/bizzkoot/copilot-tracker/commit/72aefd2f8aea44bd4d8a482e6b2b38ced863e26f))

## [2.1.0](https://github.com/bizzkoot/copilot-tracker/compare/v2.0.1...v2.1.0) (2026-02-07)

### Features

- **polling:** add dynamic background polling with lifecycle management ([f07bc53](https://github.com/bizzkoot/copilot-tracker/commit/f07bc53247f7665208a857ae5f8cd7aefd05a9e4))

### Bug Fixes

- **platform:** resolve Linux window management and improve cross-platform compatibility ([f4102f3](https://github.com/bizzkoot/copilot-tracker/commit/f4102f3adf3f4da121fc284cd8f647f7010c0a30))
- **runtime:** use tauri::async_runtime::spawn instead of tokio::spawn ([45985a3](https://github.com/bizzkoot/copilot-tracker/commit/45985a32da7f820bf652fec0b35e71b9520fb9a8))

## [2.0.1](https://github.com/bizzkoot/copilot-tracker/compare/v2.0.0...v2.0.1) (2026-02-07)

### Bug Fixes

- sync Tauri version to 2.0.0 and add automated version sync workflow ([a6d0a19](https://github.com/bizzkoot/copilot-tracker/commit/a6d0a19950fa8268423528ca1e3c73bc40bbe46d))

## [2.0.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.5.1...v2.0.0) (2026-02-07)

### ⚠ BREAKING CHANGES

- Electron builds discontinued after v1.5.1

### Features

- add customizable tray icon display formats ([6972b39](https://github.com/bizzkoot/copilot-tracker/commit/6972b39a8342c72138337f87849da9327b6228d4))
- **tray:** enhance system tray menu with consumption metrics ([6ca57f4](https://github.com/bizzkoot/copilot-tracker/commit/6ca57f4801625ee7c798fab07b64ba0df014a2ad))

### Bug Fixes

- explicitly specify TOML type and jsonpath for Cargo.toml ([33fe842](https://github.com/bizzkoot/copilot-tracker/commit/33fe842fe17c75eb132a35e5ea8d2c2e803389c5))
- sync version to 1.5.1 and improve cross-platform compatibility ([e9abcfd](https://github.com/bizzkoot/copilot-tracker/commit/e9abcfd508d1ff9eab11345ae98a236af378c5ae))

### Continuous Integration

- remove Electron builds and update release workflow ([c5aacbf](https://github.com/bizzkoot/copilot-tracker/commit/c5aacbf198a7f243add9b2161f53fe2fad00959e))

## [1.5.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.5.0...v1.5.1) (2026-02-05)

### Bug Fixes

- properly hide dock icon on macOS using set_activation_policy ([2d957d5](https://github.com/bizzkoot/copilot-tracker/commit/2d957d57b90a8786bc1ce1a24ca43968803efe10))
- **release:** correct release-please configuration for Tauri version bumping ([6ab6731](https://github.com/bizzkoot/copilot-tracker/commit/6ab6731599b2aa0c5b5542d578cd223a995cb8a8))
- resolve tray-dashboard sync, dock visibility, and startup data issues ([75262c0](https://github.com/bizzkoot/copilot-tracker/commit/75262c0af74bb5fe3cc34cd0def6b36e8dad586a))

## [1.5.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.4.2...v1.5.0) (2026-02-05)

### Features

- **tray:** implement Retina-aware text rendering with improved sharpness ([7a2b30c](https://github.com/bizzkoot/copilot-tracker/commit/7a2b30c015cd66b9ec0b85b9a40e61f15197bfe5))

## [1.4.2](https://github.com/bizzkoot/copilot-tracker/compare/v1.4.1...v1.4.2) (2026-02-04)

### Bug Fixes

- **electron:** prevent auth window flash and reload loop on Windows ([891e29a](https://github.com/bizzkoot/copilot-tracker/commit/891e29acef7304d45fc3740c263401c42ed047d0))
- **tauri:** resolve Windows and Linux build failures ([d4fd9b2](https://github.com/bizzkoot/copilot-tracker/commit/d4fd9b22afa604433137470a9ddac979e953cf00))

## [1.4.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.4.0...v1.4.1) (2026-02-04)

### Bug Fixes

- add bundle metadata for Windows Tauri build ([611d318](https://github.com/bizzkoot/copilot-tracker/commit/611d318c74e2a137d3ed4daa01f89db0b5a3f71d))
- **electron:** resolve Windows auth window and clean code - release ([fe102ab](https://github.com/bizzkoot/copilot-tracker/commit/fe102abef3939ffc179c0a259d3c53628d8dbe18))
- remove icon array from tauri.conf.json to allow auto-detection of platform icons ([9f80fe6](https://github.com/bizzkoot/copilot-tracker/commit/9f80fe6db080ea60cefa811998773c8950ea22a0))

## [1.4.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.3.1...v1.4.0) (2026-02-04)

### Features

- add multiplatform Tauri builds to CI/CD workflow ([069f461](https://github.com/bizzkoot/copilot-tracker/commit/069f461c528e659fc0a46a0f21e89cb0438c9f8c))
- **auth:** implement auth-time usage data capture via URL hash redirect ([ec981c5](https://github.com/bizzkoot/copilot-tracker/commit/ec981c5432595c62a21c2c9f75ca3fe5f9a55685))
- **auth:** implement working hidden webview for silent usage refresh ([0299bf0](https://github.com/bizzkoot/copilot-tracker/commit/0299bf0a9ab524a378465804c5540869c1a82a45))
- **docs:** add PRD for Tauri migration of Copilot Tracker ([c153360](https://github.com/bizzkoot/copilot-tracker/commit/c1533603ca5d8857e83b945f52856660c196e025))
- **refresh:** update tray refresh to trigger re-authentication ([1260fc9](https://github.com/bizzkoot/copilot-tracker/commit/1260fc957987fb620f30964f627cdeabc1ec1a85))
- **tauri:** add linting step to tauri build process ([552a9bb](https://github.com/bizzkoot/copilot-tracker/commit/552a9bb1f1f49b67a353efec1af39a10969286e4))
- **tauri:** complete feature parity with Electron implementation ([c7cbe61](https://github.com/bizzkoot/copilot-tracker/commit/c7cbe610ee15aa6e88ed982a3bfcc6784472b5c2))
- **tauri:** complete migration implementation and build support ([78b892c](https://github.com/bizzkoot/copilot-tracker/commit/78b892c95323260059d6c7dc3153d28713ee004b))
- **tauri:** migrate authentication and dashboard logic from Electron ([f24e5ac](https://github.com/bizzkoot/copilot-tracker/commit/f24e5ac85d33daabb8426af881c3163de0289400))
- **tray:** implement Electron-style tray icon with progress indicator ([07141ac](https://github.com/bizzkoot/copilot-tracker/commit/07141ac27741d5972e70f62925e6371e2d56bd67))

### Bug Fixes

- **auth:** implement custom protocol redirect for safer data extraction ([f3024ad](https://github.com/bizzkoot/copilot-tracker/commit/f3024ad7aafcee89e576951c9db22680ea59be24))
- **auth:** implement single-window extraction with custom protocol redirect and auto-navigation ([3a47db9](https://github.com/bizzkoot/copilot-tracker/commit/3a47db9590721b80a3eb6d19381b8065f97f26ec))
- **data:** implement camelCase JSON parsing and persistent history storage ([b542216](https://github.com/bizzkoot/copilot-tracker/commit/b5422162cb7b87e6b5df501f862d9cc66a0843bc))
- **reset:** fix race condition and ensure auth state properly updates ([ecc93f0](https://github.com/bizzkoot/copilot-tracker/commit/ecc93f00f2072b778119acb1106e419eb87d8ff5))
- **reset:** properly clear all data on Reset and logout frontend ([68fa4a7](https://github.com/bizzkoot/copilot-tracker/commit/68fa4a785bc2da850fad8816797cd14c1a3f4d44))
- **reset:** wire Reset button to actual backend reset function ([886e558](https://github.com/bizzkoot/copilot-tracker/commit/886e558683e841c8ef5dbf7daf21034c92e0609e))
- **tauri:** authentication flow improvements and clippy cleanup ([bf5387a](https://github.com/bizzkoot/copilot-tracker/commit/bf5387a244f8af8c52155bf4b56399f37177a2f0))
- **tauri:** resolve 504MB binary size issue ([ad9ff57](https://github.com/bizzkoot/copilot-tracker/commit/ad9ff573a27e882949b376193fff51c268f37c9b))
- **tray:** CRITICAL BUG FIX - tray listener was parsing wrong event type ([4989ed9](https://github.com/bizzkoot/copilot-tracker/commit/4989ed9512adffd6705c9290bcd9bf8341bb9e7e))
- **tray:** remove progress circle from tray icon ([c43bf01](https://github.com/bizzkoot/copilot-tracker/commit/c43bf0115562401937d5e9a740ef88b7c62df9a4))
- **tray:** synchronize tray icon with dashboard in real-time ([a919fca](https://github.com/bizzkoot/copilot-tracker/commit/a919fca1ad3423e1f1fe0b09244d44ed79af3a00))
- **ui:** resolve all 5 dashboard and tray synchronization issues ([899f7bf](https://github.com/bizzkoot/copilot-tracker/commit/899f7bff2d5be13aefc591652acef83d8a4d558b))

## [1.3.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.3.0...v1.3.1) (2026-02-03)

### Bug Fixes

- **auth:** prevent invisible Re-Login window on Windows and stop unauthenticated GitHub polling - release ([b744aac](https://github.com/bizzkoot/copilot-tracker/commit/b744aacae9e276727195f2f75e1b13aab4450118))

## [1.3.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.2.0...v1.3.0) (2026-02-01)

### Features

- **tray:** add monthly prediction banner and dashboard action - release ([a8b4081](https://github.com/bizzkoot/copilot-tracker/commit/a8b4081db3f96d78250b0d1aa593387bfb996360))

### Bug Fixes

- startup authentication and auto-minimize on launch ([6cc332e](https://github.com/bizzkoot/copilot-tracker/commit/6cc332efe68f8ace86a5e60b16253a536bbac03a))

## [1.2.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.1.2...v1.2.0) (2026-02-01)

### Features

- add update notification system with native alerts and tray indicators ([edc2666](https://github.com/bizzkoot/copilot-tracker/commit/edc2666f2d73095188e15db9b52e45029652dce7))

## [1.1.2](https://github.com/bizzkoot/copilot-tracker/compare/v1.1.1...v1.1.2) (2026-02-01)

### Bug Fixes

- sync settings UI with tray and improve canvas types ([1eff249](https://github.com/bizzkoot/copilot-tracker/commit/1eff24942d7bd3ef31459e3c501b79f7339c2510))

## [1.1.1](https://github.com/bizzkoot/copilot-tracker/compare/v1.1.0...v1.1.1) (2026-02-01)

### Bug Fixes

- upload release assets using tag_name output instead of github.ref ([4f68eb6](https://github.com/bizzkoot/copilot-tracker/commit/4f68eb6233bedd6dbc2c65ec84408b5a8cff2b65))

## [1.1.0](https://github.com/bizzkoot/copilot-tracker/compare/v1.0.0...v1.1.0) (2026-02-01)

### Features

- add asset validation script ([1afe178](https://github.com/bizzkoot/copilot-tracker/commit/1afe1785775ca3ea3a137531c2662535064d12e7))
- add environment variables configuration and dev-only logging ([f1b99c9](https://github.com/bizzkoot/copilot-tracker/commit/f1b99c9fbb42c93df6816cf4e225eb3c732ed858))

### Bug Fixes

- add macOS traffic light safe area and update dashboard icon ([22f097a](https://github.com/bizzkoot/copilot-tracker/commit/22f097acc9893d93c2f79634bb5223020fe82b70))
- configure app icons and resolve production asset loading ([fe84671](https://github.com/bizzkoot/copilot-tracker/commit/fe84671943307c7bfa0e244928bf782505b2d1a5))
- configure canvas dependency with fallback ([66f78ba](https://github.com/bizzkoot/copilot-tracker/commit/66f78baa98c6376815eedba21126776fbc9cac26))
- modernize Electron API and fix app exit crashes ([36aeaf6](https://github.com/bizzkoot/copilot-tracker/commit/36aeaf610948fb0629fda0829addd7a9a5c6b977))
- remove global keyboard shortcuts and hide app from macOS dock ([ec1c0f7](https://github.com/bizzkoot/copilot-tracker/commit/ec1c0f7f69db662001f62b15956aa80521f5754d))
- resolve history parsing and renderer display issues ([067c1f1](https://github.com/bizzkoot/copilot-tracker/commit/067c1f1323b3edf3be7723db853d53b3e496a9aa))
- **tray:** navigation to settings and optimize icon generation ([514699e](https://github.com/bizzkoot/copilot-tracker/commit/514699e596dd84fa8ae19bdac44d9de38796e9f2))

## 1.0.0 (2026-02-01)

### Features

- add asset validation script ([1afe178](https://github.com/bizzkoot/copilot-tracker/commit/1afe1785775ca3ea3a137531c2662535064d12e7))
- add environment variables configuration and dev-only logging ([f1b99c9](https://github.com/bizzkoot/copilot-tracker/commit/f1b99c9fbb42c93df6816cf4e225eb3c732ed858))

### Bug Fixes

- add macOS traffic light safe area and update dashboard icon ([22f097a](https://github.com/bizzkoot/copilot-tracker/commit/22f097acc9893d93c2f79634bb5223020fe82b70))
- configure app icons and resolve production asset loading ([fe84671](https://github.com/bizzkoot/copilot-tracker/commit/fe84671943307c7bfa0e244928bf782505b2d1a5))
- configure canvas dependency with fallback ([66f78ba](https://github.com/bizzkoot/copilot-tracker/commit/66f78baa98c6376815eedba21126776fbc9cac26))
- modernize Electron API and fix app exit crashes ([36aeaf6](https://github.com/bizzkoot/copilot-tracker/commit/36aeaf610948fb0629fda0829addd7a9a5c6b977))
- remove global keyboard shortcuts and hide app from macOS dock ([ec1c0f7](https://github.com/bizzkoot/copilot-tracker/commit/ec1c0f7f69db662001f62b15956aa80521f5754d))
- resolve history parsing and renderer display issues ([067c1f1](https://github.com/bizzkoot/copilot-tracker/commit/067c1f1323b3edf3be7723db853d53b3e496a9aa))
- **tray:** navigation to settings and optimize icon generation ([514699e](https://github.com/bizzkoot/copilot-tracker/commit/514699e596dd84fa8ae19bdac44d9de38796e9f2))
