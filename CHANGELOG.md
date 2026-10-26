# Changelog

## 0.1.0 (2026-10-26)


### Features

* **core:** convert images and videos locally with the bundled FFmpeg ([0e1a373](https://github.com/Squ1ggly/converter-desktop/commit/0e1a373e3236766691432f081c4ff8c3a7e62446))
* replace app icon ([85b70d2](https://github.com/Squ1ggly/converter-desktop/commit/85b70d22d1d59e9a9552ffda61265c66d609ece6))
* **ui:** add drag-and-drop interface for converting files ([11bd2a4](https://github.com/Squ1ggly/converter-desktop/commit/11bd2a4a3bc1cfc02fc0ece10d674b9b77ee8bab))
* **ui:** add Settings and Help pages ([a3a49cf](https://github.com/Squ1ggly/converter-desktop/commit/a3a49cf12f0bda62732e3ffbd261617ba4925b2e))
* **ui:** rebuild interface with shadcn-svelte ([6463222](https://github.com/Squ1ggly/converter-desktop/commit/64632229c58e57372bcbb4419a8fd04cb27772a0))


### Bug Fixes

* **a11y:** label the quality slider and file rows ([1c1d206](https://github.com/Squ1ggly/converter-desktop/commit/1c1d2064416e56ba44a47ba461b000296b362c34))
* allow only one running instance ([1f08cbd](https://github.com/Squ1ggly/converter-desktop/commit/1f08cbd038e462322d53caaf4c1072c342e3afc4))
* **ci:** stop the release workflow failing when no release PR changed ([#5](https://github.com/Squ1ggly/converter-desktop/issues/5)) ([e47cf5a](https://github.com/Squ1ggly/converter-desktop/commit/e47cf5a199e4580c39bed3f2b27f982bd094cb4a))
* harden the backend from the code review ([#8](https://github.com/Squ1ggly/converter-desktop/issues/8)) ([7f102f4](https://github.com/Squ1ggly/converter-desktop/commit/7f102f4ebe43f386a9ebefbb2d58a8bde57ca98b))
* **images:** keep the transparency of HEIC and AVIF inputs ([#10](https://github.com/Squ1ggly/converter-desktop/issues/10)) ([8e31c18](https://github.com/Squ1ggly/converter-desktop/commit/8e31c185944fd52420eb73a445c1a6dff954ac2e))
* **linux:** rename the bundled FFmpeg so packages don't clash with the system's ([1f55939](https://github.com/Squ1ggly/converter-desktop/commit/1f559394bdff49b6bc99ffd294a01b83f20c29b5))
* **macos:** centre window buttons in the toolbar ([f2337dd](https://github.com/Squ1ggly/converter-desktop/commit/f2337dd6122e5bb5247f2808ef8735dedd33f2f6))
* stop folder symlink loops from hanging file import ([13ef228](https://github.com/Squ1ggly/converter-desktop/commit/13ef228236a0c24c3716ca2500291331e5880857))
* **ui:** consistent selected states and a Done badge for finished files ([e92f80c](https://github.com/Squ1ggly/converter-desktop/commit/e92f80c4ce18cb848d316c031e2c83577d893a8f))
* **ui:** fix the UI findings from the code review ([#9](https://github.com/Squ1ggly/converter-desktop/issues/9)) ([ab8ccda](https://github.com/Squ1ggly/converter-desktop/commit/ab8ccda5d3300284e9791cd46baa22e3f48fe5cc))
* **ui:** keep save location when the folder dialog is cancelled ([f534402](https://github.com/Squ1ggly/converter-desktop/commit/f53440219e79ed4732e70245363193cd3a9fec18))
* **ui:** native title on macOS, no duplicate title elsewhere ([a1317d6](https://github.com/Squ1ggly/converter-desktop/commit/a1317d666801a57efd3caca73bd972e1a1f570f0))
* **ui:** tidy Help page code and heading styles ([e1f40b0](https://github.com/Squ1ggly/converter-desktop/commit/e1f40b0e06206b13a96aa170de6203fb4f467ec4))
* **video:** apply resolution to the shorter side and never raise frame rate ([53b6a4a](https://github.com/Squ1ggly/converter-desktop/commit/53b6a4a2aa232cab37ee88c62a1ebad81a69056b))
