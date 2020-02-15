import spacy
import json
import argparse
from spacy.tokens import Doc, DocBin

from similarity import generate_models

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

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Reads json file created by the ad collector, preprocesses the ads, and generate NLP models")
    parser.add_argument("DATA_PATH", help="Path to json file that ad collector created", nargs=1)
    parser.add_argument("MODEL_DIR", help="Directory where NLP models will be stored", nargs=1)
    parser.add_argument("--num_topics", help="Number of topics that the LSI model should look for", default=15, type=int)
    parser.add_argument("--doc2vec_epochs", help="Number of epochs to train doc2vec model", default=40, type=int)
    parser.add_argument("--doc2vec_workers", help="Number of epochs to train doc2vec model", default=1, type=int)
    args = parser.parse_args()

    DATASET_LOCATION = args.DATA_PATH[0]
    MODEL_SAVE_LOCATION = args.MODEL_DIR[0]
    num_topics = args.num_topics
    doc2vec_epochs = args.doc2vec_epochs
    doc2vec_workers = args.doc2vec_workers
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
        generate_models(raw_corpus, processed_corpus, MODEL_SAVE_LOCATION, num_topics, doc2vec_epochs, doc2vec_workers)