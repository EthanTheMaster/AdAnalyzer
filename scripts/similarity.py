from gensim import corpora
from gensim import models
from gensim import similarities
from gensim.matutils import hellinger
from gensim.models.doc2vec import Doc2Vec, TaggedDocument

import os
import json
import argparse

CORPUS_FILE_NAME = "corpus_data.json"
DICTIONARY_FILE_NAME = "dictionary.ser"
BOW_CORPUS_FILE_NAME = "bow.corpus"
TFIDF_CORPUS_FILE_NAME = "tfidf.corpus"
TFIDF_MODEL_FILE_NAME = "tfidf.model"
LSI_CORPUS_FILE_NAME = "lsi.corpus"
LSI_MODEL_FILE_NAME = "lsi.model"
DOC2VEC_MODEL_FILE_NAME = "doc2vec.model"

def generate_models(raw_corpus, processed_corpus, model_save_dir, topic_num, doc2vec_epochs=40, doc2vec_workers=1):
    # Make folder directory if it does not exist
    if not os.path.exists(model_save_dir):
        os.makedirs(model_save_dir)

    # Save model data
    with open(os.path.join(model_save_dir, CORPUS_FILE_NAME), "w") as f:
        json.dump({"raw_corpus": raw_corpus, "processed_corpus": processed_corpus, "topic_num": topic_num}, f)

    # Generate the corpus dictionary and save it
    dictionary = corpora.Dictionary(processed_corpus)
    dictionary.save(os.path.join(model_save_dir, DICTIONARY_FILE_NAME))

    # Generate bag of word corpus and save the corpus
    corpus_bow = list(map(lambda text: dictionary.doc2bow(text), processed_corpus))
    corpora.MmCorpus.serialize(os.path.join(model_save_dir, BOW_CORPUS_FILE_NAME), corpus_bow)

    # Create the TFIDF model and save it 
    tfidf = models.TfidfModel(corpus_bow)
    corpus_tfidf = tfidf[corpus_bow]
    corpora.MmCorpus.serialize(os.path.join(model_save_dir, TFIDF_CORPUS_FILE_NAME), corpus_tfidf)
    tfidf.save(os.path.join(model_save_dir, TFIDF_MODEL_FILE_NAME))

    # Create LSI model and save it
    lsi = models.LsiModel(corpus_tfidf, id2word=dictionary, num_topics=topic_num)
    corpus_lsi = lsi[corpus_tfidf]
    corpora.MmCorpus.serialize(os.path.join(model_save_dir, LSI_CORPUS_FILE_NAME), corpus_lsi)
    lsi.save(os.path.join(model_save_dir, LSI_MODEL_FILE_NAME))

    # Train Doc2Vec model on corpus
    documents = [TaggedDocument(doc, [i]) for i, doc in enumerate(processed_corpus)]
    doc2vec_model = Doc2Vec(documents, vector_size=50, epochs=doc2vec_epochs, workers=doc2vec_workers)
    doc2vec_model.save(os.path.join(model_save_dir, DOC2VEC_MODEL_FILE_NAME))


def find_similar_docs(model_save_dir, doc_id, num_best):
    corpus_data = json.load(open(os.path.join(model_save_dir, CORPUS_FILE_NAME)))
    raw_corpus = corpus_data["raw_corpus"]
    processed_corpus = corpus_data["processed_corpus"]

    dictionary = corpora.Dictionary.load(os.path.join(model_save_dir, DICTIONARY_FILE_NAME))

    corpus_bow = corpora.MmCorpus(os.path.join(model_save_dir, BOW_CORPUS_FILE_NAME))
    corpus_tfidf = corpora.MmCorpus(os.path.join(model_save_dir, TFIDF_CORPUS_FILE_NAME))
    corpus_lsi = corpora.MmCorpus(os.path.join(model_save_dir, LSI_CORPUS_FILE_NAME))

    tfidf = models.TfidfModel.load(os.path.join(model_save_dir, TFIDF_MODEL_FILE_NAME))
    lsi = models.LsiModel.load(os.path.join(model_save_dir, LSI_MODEL_FILE_NAME))
    doc2vec_model = Doc2Vec.load(os.path.join(model_save_dir, DOC2VEC_MODEL_FILE_NAME))

    tfidf_index = similarities.SparseMatrixSimilarity(corpus_tfidf, num_best=num_best)
    lsi_index = similarities.MatrixSimilarity(corpus_lsi, num_best=num_best)

    # Create vector representation of document
    bow_vec = dictionary.doc2bow(processed_corpus[doc_id])
    tfidf_vec = tfidf[bow_vec]
    # Find most similar with tfidf, lsi, doc2vec models
    tfidf_most_sim = tfidf_index[tfidf_vec]
    lsi_most_sim = lsi_index[lsi[tfidf_vec]]

    doc2vec_vector = doc2vec_model.infer_vector(processed_corpus[doc_id])
    doc2vec_most_sim = doc2vec_model.docvecs.most_similar([doc2vec_vector], topn=num_best)

    # Currently not serializable as numpy int64 is not JSON serializable
    make_serializable = lambda pair: (int(pair[0]), pair[1])
    # Zip all 3 results together
    return list(\
                zip(\
                    map(make_serializable, tfidf_most_sim),\
                    map(make_serializable, lsi_most_sim),\
                    map(make_serializable, doc2vec_most_sim)\
                )\
            )

def find_interesting_words(model_save_dir, num_best):
    dictionary = corpora.Dictionary.load(os.path.join(model_save_dir, DICTIONARY_FILE_NAME))
    corpus_tfidf = corpora.MmCorpus(os.path.join(model_save_dir, TFIDF_CORPUS_FILE_NAME))

    # Find interesting words based on the tfidf score
    words_score = {}
    for doc in corpus_tfidf:
        for pair in doc:
            # Don't add words to the list if they have a space ... named entities
            if not("_" in dictionary[pair[0]]):
                words_score[pair[0]] = words_score.get(pair[0], 0) + pair[1]
    # Return list of words by matching word id to dictionary
    return [dictionary[i] for i in sorted(words_score, key=lambda k: words_score[k], reverse=True)[:num_best]]

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Utilty script to find interesting words in a corpus and to find similar documents in a corpus")
    subparsers = parser.add_subparsers(dest="which")
    subparsers.required = True

    parser_interesting_words = subparsers.add_parser("interesting_words", help="Finds words of interest from a corpus")
    parser_interesting_words.add_argument("MODEL_PATH", help="Directory holding NLP models generated after preprocessing", nargs=1)
    parser_interesting_words.add_argument("--num_best", help="The number of words that should be returned", type=int, default=15)

    parser_similar_docs = subparsers.add_parser("similar_docs", help="Finds similar documents in the corpus")
    parser_similar_docs.add_argument("MODEL_PATH", help="Directory holding NLP models generated after preprocessing", nargs=1)
    parser_similar_docs.add_argument("DOC_ID", help="The number of words that should be returned", nargs=1, type=int)
    parser_similar_docs.add_argument("--num_best", help="The number of documents from each model that should be returned", type=int, default=15)

    args = parser.parse_args()
    if args.which == "interesting_words":
        print(json.dumps(find_interesting_words(args.MODEL_PATH[0], args.num_best)))
    elif args.which == "similar_docs":
        print(json.dumps(find_similar_docs(args.MODEL_PATH[0], args.DOC_ID[0], args.num_best)))