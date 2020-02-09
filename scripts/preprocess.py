import spacy
from spacy.tokens import Doc, DocBin

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
    normalized_tokens = map(lambda token: token.lemma_.lower().replace(" ", "_"), filtered_tokens)
    return list(normalized_tokens)
    

DATASET_LOCATION = "/home/ethanlam/Desktop/LearnProgramming/AdAnalyzer/Bloomberg2-4-2020_CLEAN"

# Load Language Model
nlp = spacy.load("en_core_web_md", disable=["tagger", "parser"])
merge_entity = nlp.create_pipe("merge_entities")
nlp.add_pipe(merge_entity)

# Generate Annotation for each line ... assume each line of file is a document
Doc.set_extension("preprocessed_tokens", default=[])
for doc in nlp.pipe(open(DATASET_LOCATION)):
    doc._.preprocessed_tokens = preprocess(doc)
    print(doc._.preprocessed_tokens)