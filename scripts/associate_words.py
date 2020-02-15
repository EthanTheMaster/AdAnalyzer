import spacy
from spacy import displacy
from spacy.tokens import Doc

import json
import argparse
import os

class AssociationNode:
    def __init__(self, word):
        self.word = word
        # Maps a string to a list of indices (these indices point to documents in the corpus)
        self.edges = {}


def get_node(graph, word):
    if not(word in graph):
        graph[word] = AssociationNode(word)
    return graph[word]

def connect_nodes(node1, node2, doc_idx):
    if not(node2.word in node1.edges):
        node1.edges[node2.word] = set()
    if not(node1.word in node2.edges):
        node2.edges[node1.word] = set()
    
    node1.edges[node2.word].add(doc_idx)
    node2.edges[node1.word].add(doc_idx)

# Given a token, return a list of all other tokens connected to input token via a conj dependency
def get_associated_conj_terms(token):
    # Perform DFS
    open_list = [token]
    res = []
    while len(open_list) > 0:
        res.append(open_list.pop())
        for child in res[-1].children:
            if child.dep_ == "conj":
                open_list.append(child)
    return res

def add_edges(graph, token, doc_idx):
    word = token.lemma_.lower()
    # Ignore list
    if word in ["the", "a"]:
        return
    if token.pos_ == "AUX" and token.dep_ == "AUX":
        return
    
    # TODO: Deal with once corefence can be resolved ...
    if token.pos_ == "PRON":
        return
    if token.pos_ in ["NOUN", "PROPN", "ADJ"]:
        if token.dep_ in ["nsubj", "nsubjpass"]:
            # Associate all subjects with all verbs
            subj_tokens = get_associated_conj_terms(token)
            verb_tokens = get_associated_conj_terms(token.head)

            for subj in subj_tokens:
                for verb in verb_tokens:
                    subj_node = get_node(graph, subj.lemma_.lower())
                    verb_node = get_node(graph, verb.lemma_.lower())
                    # Don't associate subject with auxiliary verb
                    if verb.pos_ == "AUX":
                        continue
                    # TODO: Deal with once corefence can be resolved ...
                    if subj.pos_ == "PRON":
                        continue
                    connect_nodes(subj_node, verb_node, doc_idx)
        elif token.dep_ == "attr":
            # Associate all attributes to all subjects
            attr_tokens = get_associated_conj_terms(token)

            subj_tokens = []
            for child in token.head.children:
                if child.dep_ == "nsubj":
                    subj_tokens = get_associated_conj_terms(child)
                    break

            for attr in attr_tokens:
                for subj in subj_tokens:
                    attr_node = get_node(graph, attr.lemma_.lower())
                    subj_node = get_node(graph, subj.lemma_.lower())
                    # TODO: Deal with once corefence can be resolved ...
                    if subj.pos_ == "PRON":
                        continue
                    connect_nodes(attr_node, subj_node, doc_idx)
        elif token.dep_ == "compound":
            # Associate word with compound token
            node1 = get_node(graph, word)
            node2 = get_node(graph, token.head.lemma_.lower())
                
            connect_nodes(node1, node2, doc_idx)
        elif token.dep_ == "dobj":
            # Associate direct objects (DOs) with verbs along with DOs and subjects
            do_tokens = get_associated_conj_terms(token)
            subj_tokens = []
            # Traverse up the conj chain to main verb to extract subjects .. Eg "Alice eat the burger and drinks the soda."
            current_verb = token.head
            while current_verb.dep_ == "conj":
                current_verb = current_verb.head
            for child in current_verb.children:
                if child.dep_ == "nsubj":
                    subj_tokens = get_associated_conj_terms(child)
                    break

            # Associate DOs with verb
            verb_node = get_node(graph, token.head.lemma_.lower())
            for do in do_tokens:
                do_node = get_node(graph, do.lemma_.lower())
                connect_nodes(do_node, verb_node, doc_idx)
            # Associate DOs with subjects
            for do in do_tokens:
                for subj in subj_tokens:
                    do_node = get_node(graph, do.lemma_.lower())
                    subj_node = get_node(graph, subj.lemma_.lower())
                    # TODO: Deal with once corefence can be resolved ...
                    if subj.pos_ == "PRON":
                        continue
                    connect_nodes(do_node, subj_node, doc_idx)
        elif token.dep_ == "dative":
            # Associate dative with DOs
            dative_tokens = get_associated_conj_terms(token)
            do_tokens = []
            for child in token.head.children:
                if child.dep_ == "dobj":
                    do_tokens = get_associated_conj_terms(child)
                    break
            for dative in dative_tokens:
                for do in do_tokens:
                    dative_node = get_node(graph, dative.lemma_.lower())
                    do_node = get_node(graph, do.lemma_.lower())
                    # TODO: Deal with once corefence can be resolved ...
                    if do.pos_ == "PRON":
                        continue
                    connect_nodes(dative_node, do_node, doc_idx)
        elif token.dep_ == "pobj":
            # Associate pobj with verbs along with pobj and subjects (if such a relation exists)
            pobj_tokens = get_associated_conj_terms(token)
            subj_tokens = []
            # Structure being matched: VERB -> PREP -> [VERB/NOUN]
            if token.head.head.pos_ == "VERB":
                # Traverse up the conj chain to main verb to extract subjects
                current_verb = token.head.head
                while current_verb.dep_ == "conj":
                    current_verb = current_verb.head
                for child in current_verb.children:
                    if child.dep_ == "nsubj":
                        subj_tokens = get_associated_conj_terms(child)
                        break

                # Associate pobjs with verb
                verb_node = get_node(graph, token.head.head.lemma_.lower())
                for pobj in pobj_tokens:
                    pobj_node = get_node(graph, pobj.lemma_.lower())
                    connect_nodes(pobj_node, verb_node, doc_idx)
                # Associate pobjs with subjects
                for pobj in pobj_tokens:
                    for subj in subj_tokens:
                        pobj_node = get_node(graph, pobj.lemma_.lower())
                        subj_node = get_node(graph, subj.lemma_.lower())
                        # TODO: Deal with once corefence can be resolved ...
                        if subj.pos_ == "PRON":
                            continue
                        connect_nodes(pobj_node, subj_node, doc_idx)
            elif token.head.head.pos_ == "NOUN":
                noun_node = get_node(graph, token.head.head.lemma_.lower())
                for pobj in pobj_tokens:
                    pobj_node = get_node(graph, pobj.lemma_.lower())
                    connect_nodes(pobj_node, noun_node, doc_idx)
                    
        elif token.dep_ == "acomp":
            # Associate all adj complements to all subjects
            acomp_tokens = get_associated_conj_terms(token)

            subj_tokens = []
            for child in token.head.children:
                if child.dep_ == "nsubj":
                    subj_tokens = get_associated_conj_terms(child)
                    break

            for acomp in acomp_tokens:
                for subj in subj_tokens:
                    acomp_node = get_node(graph, acomp.lemma_.lower())
                    subj_node = get_node(graph, subj.lemma_.lower())
                    # TODO: Deal with once corefence can be resolved ...
                    if subj.pos_ == "PRON":
                        continue
                    connect_nodes(acomp_node, subj_node, doc_idx)
        elif token.dep_ in ["amod", "nmod", "poss", "advcl"]:
            # Associate each modifier with modifiee
            modifier_tokens = get_associated_conj_terms(token)
            modifiee_node = get_node(graph, token.head.lemma_.lower())
            for modifier in modifier_tokens:
                modifier_node = get_node(graph, modifier.lemma_.lower())
                connect_nodes(modifier_node, modifiee_node, doc_idx)
    elif token.pos_ in ["VERB", "ADV"]:
        if token.dep_ in ["ccomp", "xcomp", "advmod"]:
            # Associate clausal complement(s) or complementing adverbs with the verb
            comp_tokens = get_associated_conj_terms(token)
            complementee_node = get_node(graph, token.head.lemma_.lower())
            for comp in comp_tokens:
                comp_node = get_node(graph, comp.lemma_.lower())
                connect_nodes(comp_node, complementee_node, doc_idx)
        elif token.dep_ == "relcl":
            # Associate each relative clause verb with the "subject"
            relcl_tokens = get_associated_conj_terms(token)
            subj_tokens = get_associated_conj_terms(token.head)
            for relcl in relcl_tokens:
                for subj in subj_tokens:
                    relcl_node = get_node(graph, relcl.lemma_.lower())
                    subj_node = get_node(graph, subj.lemma_.lower())
                    # TODO: Deal with once corefence can be resolved ...
                    if subj.pos_ == "PRON":
                        continue
                    connect_nodes(relcl_node, subj_node, doc_idx)
            
def gen_association_graph(docs):
    # Maps strings (words) to node
    graph = {}
    for doc_idx, doc in enumerate(docs):
        for token in doc:
            add_edges(graph, token, doc_idx)

    return graph

def gen_json(graph, file_obj):
    edges = {}
    for key in graph:
        edges[key] = [{"word":target, "docs":[idx for idx in graph[key].edges[target]]} for target in graph[key].edges]
    return json.dump(edges, file_obj)

parser = argparse.ArgumentParser(description="Reads json file generated by ad collector and generates an file holding association graph data")
parser.add_argument("DATA_PATH", help="Path to the corpus data json file generated during the preprocess and model generation stage", nargs=1)
parser.add_argument("SAVE_DIR", help="Directory to save association graph file in", nargs=1)

args = parser.parse_args()
data_location = args.DATA_PATH[0]
save_location = args.SAVE_DIR[0]
GRAPH_FILE_NAME = "association_graph.json"

with open(data_location) as f:
    nlp = spacy.load("en_core_web_md")
    corpus_data = json.load(f)
    graph = gen_association_graph(nlp.pipe(corpus_data["raw_corpus"]))
    
    # Make folder directory if it does not exist
    if not os.path.exists(save_location):
        os.makedirs(save_location)
    with open(os.path.join(save_location, GRAPH_FILE_NAME), "w") as graph_file:
        gen_json(graph, graph_file)

