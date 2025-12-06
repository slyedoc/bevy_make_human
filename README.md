# bevy_make_human

Customizable humanoid characters in Bevy.

[make huamn](https://static.makehumancommunity.org/index.html)
[mpfb2 repo](https://github.com/makehumancommunity/mpfb2.git)

There is also [Humentity](https://github.com/emberlightstudios/Humentity) that you should check out.

## TLDR; 

Requires git lfs to clone this repo due to  assets.

```bash
Unlike most bevy plugins, you will need to copy assets/make_human from this repo to your own project's assets folder to use this.

## Asset Enums

I liked the idea of supporting any assets just by letting users drop them in the right folders and a decade plus of community assets, tuturials and tools that bevy users can utilize.  Having to use string paths to access assets is error prone and lead to runtime errors instead of compile time.

To that end I created a [build.rs](build.rs) that scans the assets folders and generates enums for the assets it finds.  This way users can still drop in any assets they want and the code will update to reflect that(yes this prevents things like dynamic user generated content in realtime, but thats not a problem I have):

It looks for: ```assets/make_human```, supports BEVY_ASSET_ROOT.

### Adding Assets

1. Download asset packs from [MakeHuman Community Asset Packs](https://static.makehumancommunity.org/assets/assetpacks.html) and unzip them into your `assets/make_human/` folder. 
2. Add .meta files if needed, compair with exiting assets of simaler type to see what needs .meta files are needed.
  - TODO: document and make script for this
3. Build, and the enums will update to reflect the new assets.

### Current Assets 

Assets are from mpfb2 repo asset data and make human community [Asset Packs](https://static.makehumancommunity.org/assets/assetpacks.html):

```
animal01, animal02, animal03, animal04, arms01, bodyparts01, bodyparts02, bodyparts03, bodyparts04, bodyparts05, bodyparts06, cheek01, dress01, dress02, dress03, ears01, equipment01, equipment02, equipment03, eyebrows01, eyelashes01, glasses01, glasses02, gloves01, hair01, hair02, hair03, hands01, hats01, hats02, hats03, hats04, masks01, masks02, nose01, pants01, pants02, pants03, poses01, poses02, shirts01, shirts02, shirts03, shoes01, shoes02, shoes03, skins01, skins02, skins03, skirts01, skirts02, suits01, suits02, suits03, system_clothes_materials01, system_hair_materials01, underwear01, underwear02, underwear03, underwear04, makehuman_system_assets
```

I will most likely remove most these and give instructions to download and zip but wanted to get a large variety to test everything.

## TODO

  - [ ] Script to create meta files

  - [ ] Animations
    - [ ] Aabb
    - [ ] Walk cycles
    - [ ] Facial rigs
  - [ ] [Delete Groups](https://static.makehumancommunity.org/assets/creatingassets/makeclothes/makeclothes_deletegroups.html)
  - [ ] Allow for morphes via Shared meshes, currently applying morphies by constructing new mesh, so in effect every mesh is unique.  Materials should still be be shared though.
    
## License

All assets come from make human community and mpfb2 and are licensed under:

* CC0 1.0 Universal [LICENSE-CCO](LICENSE-CCO)
 see [mpfb2](https://github.com/makehumancommunity/mpfb2/blob/master/LICENSE.md):

Rest of the code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
Code is licensed under MIT or Apache-2.0 at your option.
