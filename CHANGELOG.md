# Changelog

All notable changes to this project will be automatically documented in this file.
## [0.3.1] - 2026-02-23

### Build

- **napi**: Upgrade to napi-rs v3 and update build configuration([5998665](https://github.com/usemarbles/langmail/commit/5998665678c296f740a64530a354e4c0420f45e3))

### CI

- Update napi-rs build and artifact commands([6cc56e4](https://github.com/usemarbles/langmail/commit/6cc56e4af80c1cf25b1becb009e3f597728433c3))
- Update napi create-npm-dirs command syntax([904e096](https://github.com/usemarbles/langmail/commit/904e0963a7339a6a8ab3d005e27228ab1f2303d5))

### Miscellaneous

- Regenerate index.js with version 0.3.1([5778983](https://github.com/usemarbles/langmail/commit/57789837c3dc960fe1f101b674d12ca37b22eea0))
- Bump version to 0.3.1([18e282b](https://github.com/usemarbles/langmail/commit/18e282b1d427cab8ce781e8ca5bf3fdbf45a39b0))
## [0.3.0] - 2026-02-23

### Bug Fixes

- Commit index.js to repo, remove CI artifact dance([76e4841](https://github.com/usemarbles/langmail/commit/76e484149f5bd6708a0e627125f723527a2afa68))

### CI

- Grant write permission to pull-requests in code review workflow([7162afe](https://github.com/usemarbles/langmail/commit/7162afe73465a1fdded2f6137f1a541e921c94da))
- Add generated files staleness check([422de13](https://github.com/usemarbles/langmail/commit/422de134d646c7fefedc26e34fb5fbd8b4d6d7b9))

### Features

- Feat(core): add call-to-action extraction with json-ld and heuristic
scoring([3adc0f9](https://github.com/usemarbles/langmail/commit/3adc0f935909966bbd8d2e0f0bca7278d6503a12))

### Miscellaneous

- **langmail**: Bump platform-specific dependencies to 0.3.0([25e8ce6](https://github.com/usemarbles/langmail/commit/25e8ce6c065512cfc621ac10997e673081a59627))
- Bump version to 0.3.0([28ae8f0](https://github.com/usemarbles/langmail/commit/28ae8f0a6fea35a404443a13485720fd9de415f8))

### Refactor

- Rename message_id to rfc_message_id for clarity([9c18ed8](https://github.com/usemarbles/langmail/commit/9c18ed8edcdd47897694528d37a8b1fad5434c21))

### Testing

- Replace personally identifiable information in fixtures([d1a2594](https://github.com/usemarbles/langmail/commit/d1a259477e02526ee828ca455eb58529ee1ce247))
## [0.2.2] - 2026-02-23

### Miscellaneous

- Bump version to 0.2.2 and include js binding in publish([504701f](https://github.com/usemarbles/langmail/commit/504701f4977036502858722b67a74ded1d6920f0))
## [0.2.1] - 2026-02-23

### Bug Fixes

- Package name([05cb868](https://github.com/usemarbles/langmail/commit/05cb868a9044e17fd4e1dac93185c43311d38b12))

### Miscellaneous

- Bump version to 0.2.1([bb43ccb](https://github.com/usemarbles/langmail/commit/bb43ccb0cf779b22e3d6ba1e0495b853ff9dc1e8))
## [0.2.0] - 2026-02-20

### Bug Fixes

- Do not strip forwarded messages([3711bc1](https://github.com/usemarbles/langmail/commit/3711bc145218e8231ccb2871897bb195c2d2a7a1))
- Strip tables([07deed1](https://github.com/usemarbles/langmail/commit/07deed198e95a8850c02264a6461aa0db5211b6d))
- Remove leading space([a97a5f2](https://github.com/usemarbles/langmail/commit/a97a5f252b4fae885dc6033f4ff9c5209002a61f))
- Time formatting([a7757c0](https://github.com/usemarbles/langmail/commit/a7757c0b182591d2f6f5a4082533e135298ae0a3))
- Bullet points and numbered lists([206170d](https://github.com/usemarbles/langmail/commit/206170d65460ceb469b56a1444218b11f398ab5c))
- **npm**: Update test script to run node tests directly([41eb55c](https://github.com/usemarbles/langmail/commit/41eb55c6ff9d76e5594d0fb483e9c0eb1240c45a))
- **scripts**: Improve anonymizer regex robustness and QP handling([92fba4c](https://github.com/usemarbles/langmail/commit/92fba4cc0e3343f2d27531eaae7c65738a65442e))
- **scripts**: Handle MIME epilogue and avoid global regex footgun in anonymizer([c3512e2](https://github.com/usemarbles/langmail/commit/c3512e286bbf151c25c9ddec3500d868ccab9806))
- **scripts**: Improve robustness of email anonymization script([f131601](https://github.com/usemarbles/langmail/commit/f131601bfe8bd5e11972f3ac833a008a47bab3db))
- **types**: Add deprecated EmailOutput type alias for backwards compatibility([4510535](https://github.com/usemarbles/langmail/commit/4510535eb06a182412ebcc9660f00c1f3430ce99))

### Documentation

- Increase version([05d646d](https://github.com/usemarbles/langmail/commit/05d646d731c8a34a45b93196daba2d616ca7bbf2))

### Features

- Properly format hyperlinks as markdown([4fe9db9](https://github.com/usemarbles/langmail/commit/4fe9db9b289467cd375dff237f991a0afc9c95c6))
- **tests**: Add amie promo email fixture for testing([28fdf84](https://github.com/usemarbles/langmail/commit/28fdf840c37d1c68c6351fbfcc60a720ee86791a))
- **html,core**: Improve html entity decoding and body extraction([959adb5](https://github.com/usemarbles/langmail/commit/959adb520d4a1965a4e534e0112cb76a5c187a21))
- **core**: Improve text cleaning and whitespace handling([b565e98](https://github.com/usemarbles/langmail/commit/b565e98c636ba0de4d805956989758264d274211))
- **html**: Add html entity stripping for zero-width characters([9f56248](https://github.com/usemarbles/langmail/commit/9f56248a5b8a3a8d2d764cb235d9042cd812574a))
- **core**: Add zero-width character stripping([0aa332d](https://github.com/usemarbles/langmail/commit/0aa332db898161244455c3ad508401ab211ac69a))
- **types**: Add `toLlmContext` function to format email for llm prompts([db7f6ae](https://github.com/usemarbles/langmail/commit/db7f6ae8effc9786d226c99e3d3a156ca70b0534))
- **types**: Add `to_llm_context` method for email processing([2d94aa2](https://github.com/usemarbles/langmail/commit/2d94aa2de943e5f7bb2f0ddb4bf6f3a7935565ae))
- **scripts**: Enhance email anonymization for better PII handling([4506ed8](https://github.com/usemarbles/langmail/commit/4506ed84b7a7e37ca587084a90ff77bb11f201d1))
- **core**: Enhance qp text anonymization with preserving line breaks([a9059e9](https://github.com/usemarbles/langmail/commit/a9059e9fa7a654b43a4d58e1b84000a21b62dc60))
- **anonymize**: Enhance pii anonymization in email fixtures([a26f85d](https://github.com/usemarbles/langmail/commit/a26f85d15b728084042cba48dbc1c098acb729a4))
- **scripts**: Add email fixture anonymization script([b31e47c](https://github.com/usemarbles/langmail/commit/b31e47ce5bd06ba935aa1656808138f6f67a67eb))

### Operations

- Drop aarch64-unknown-linux-musl build target for now([b767603](https://github.com/usemarbles/langmail/commit/b767603a1ab52726f2c2a8a1b12a5bb32dab4f1c))
- Update Rust in musl Docker build step([dbb22f8](https://github.com/usemarbles/langmail/commit/dbb22f87fed6e635da83936c1028c68740ed0604))
- Fix napi build command to use --cargo-cwd flag([f66cbae](https://github.com/usemarbles/langmail/commit/f66cbaef6d68eeea29771c9c58a0902add04a542))
- Improve publishing workflow([f065b0a](https://github.com/usemarbles/langmail/commit/f065b0a86978d9f22dac6dbdfbeedfa5d82cb0a9))

### Refactor

- Formatting([fd33ed3](https://github.com/usemarbles/langmail/commit/fd33ed377f015c7c467230cac758025e8f41a865))
- Remove unnecessary blank line at end of `lib.rs`([79356be](https://github.com/usemarbles/langmail/commit/79356be9eb6ea8e550cf13044563070b2c2894f7))
- Refactor: improve whitespace and invisible character handling in email
preprocessing([8a6f1fa](https://github.com/usemarbles/langmail/commit/8a6f1fa83b62ca6d271bfcd9c49a22816f06f52c))
- Rename `EmailOutput` to `ProcessedEmail`([7ca0dd3](https://github.com/usemarbles/langmail/commit/7ca0dd346da11fc1c7a1f424217df5b39efcf3d4))
- Improve code formatting and iterator usage([14fede1](https://github.com/usemarbles/langmail/commit/14fede11cbed36b7d6129ce1a1e3aca6f0f8102f))

### Testing

- **linkedin**: Add test fixtures and parsing for linkedin emails([c294d4c](https://github.com/usemarbles/langmail/commit/c294d4c7dbfc96499217e6346a07331520bdd60f))
- **core**: Add linkedin email fixture([1790952](https://github.com/usemarbles/langmail/commit/1790952b6d8cc70328e9efa3506384e68ee5fe9b))

