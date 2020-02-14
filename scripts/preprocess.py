import spacy
import json
from spacy.tokens import Doc, DocBin

from similarity import generate_models, load_models

def preprocess_filter(token):
    if token.is_punct:
        return False
    if token.is_space:
        return False
    if token.is_stop:
        return False
    if token.ent_type_ in ["DATE", "MONEY", "TIME", "PERCENT", "MONEY", "QUANTITY", "CARDINAL"]:
        return False
    return True

# Returns a list of words that are preprocessed for future document analysis
def preprocess(doc):
    filtered_tokens = filter(preprocess_filter, doc)
    # normalized_tokens = map(lambda token: token.lemma_.lower().replace(" ", "_"), filtered_tokens)
    normalized_tokens = []
    for token in filtered_tokens:
        # Combine named entities together
        normalized_tokens.append(token.lemma_.lower().replace(" ", "_"))
        # Split multi-word entities into each word and add to preprocessed text
        token_split = token.text.lower().split(" ")
        if len(token_split) > 1:
            for subword in token_split:
                normalized_tokens.append(subword)
    return list(normalized_tokens)
    

DATASET_LOCATION = "data/bloomberg_ad_stats.json"

# Load Language Model
nlp = spacy.load("en_core_web_md", disable=["tagger", "parser"])
merge_entity = nlp.create_pipe("merge_entities")
nlp.add_pipe(merge_entity)

with open(DATASET_LOCATION) as dataset:
    raw_corpus = []
    processed_corpus = []
    for idx, doc in enumerate(nlp.pipe(json.load(dataset))):
        raw_corpus.append(doc.text)
        processed_corpus.append(preprocess(doc))
    generate_models(raw_corpus, processed_corpus, "models", 15)