import spacy

# Explore how words of a certain part of speech is used ... this is for the annotation and information extraction
corpus_file = "/home/ethanlam/Desktop/LearnProgramming/AdAnalyzer/StateOfUnion"
nlp = spacy.load("en_core_web_md")
for doc in nlp.pipe(open(corpus_file)):
    for token in doc:
        if token.pos_ == "ADV":
            print('"' + token.sent.text + '": ' + token.text + "(" + str(token.is_stop) + ") => " + token.dep_)
