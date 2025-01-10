use tokio::task::JoinSet;

use crate::{
    cli::Opts,
    error::Result,
    wordlist::{
        transformation::{Transformer, WordlistTransformerRegistry},
        Wordlist,
    },
};

pub struct WordlistProcessor<'a> {
    opts: &'a Opts,
}

impl<'a> WordlistProcessor<'a> {
    pub fn new(opts: &'a Opts) -> Self {
        Self { opts }
    }

    pub async fn process_wordlists(&self) -> Result<Vec<Wordlist>> {
        let transformer = self.create_transformer()?;
        let mut set = JoinSet::new();

        self.spawn_wordlist_tasks(&mut set, &transformer);
        self.collect_results(set).await
    }

    fn create_transformer(&self) -> Result<Transformer<String>> {
        let transformers = self
            .opts
            .transforms
            .iter()
            .map(|(name, arg)| WordlistTransformerRegistry::construct(name, arg.as_deref()))
            .collect::<Result<Vec<_>>>()?;

        Ok(Transformer::new(transformers))
    }

    fn spawn_wordlist_tasks(
        &self,
        set: &mut JoinSet<Result<Wordlist>>,
        transformer: &Transformer<String>,
    ) {
        for path in &self.opts.wordlists {
            let path = path.clone();
            let transformer = transformer.clone();

            set.spawn(async move {
                let mut wordlist = Wordlist::from_path(&path).await?;
                wordlist.dedup();
                wordlist.transform(&transformer);
                Ok(wordlist)
            });
        }
    }

    async fn collect_results(&self, mut set: JoinSet<Result<Wordlist>>) -> Result<Vec<Wordlist>> {
        let mut wordlists = Vec::new();
        while let Some(result) = set.join_next().await {
            wordlists.push(result??);
        }
        Ok(wordlists)
    }
}
