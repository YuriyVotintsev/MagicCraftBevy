use std::marker::PhantomData;

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::actors::MobsBalance;
use crate::balance::GameBalance;
use crate::particles::ParticleConfigRaw;

pub trait RonAsset: Asset + for<'de> Deserialize<'de> {
    const EXTENSION: &'static str;
}

#[derive(TypePath)]
pub struct RonAssetLoader<A: RonAsset>(PhantomData<A>);

impl<A: RonAsset> Default for RonAssetLoader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A: RonAsset> AssetLoader for RonAssetLoader<A> {
    type Asset = A;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        Ok(ron::from_str(content)?)
    }

    fn extensions(&self) -> &[&str] {
        &[A::EXTENSION]
    }
}

impl RonAsset for GameBalance {
    const EXTENSION: &'static str = "balance.ron";
}

impl RonAsset for MobsBalance {
    const EXTENSION: &'static str = "mobs.ron";
}

impl RonAsset for ParticleConfigRaw {
    const EXTENSION: &'static str = "particle.ron";
}
