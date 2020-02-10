import json

import networkx as nx
import matplotlib.pyplot as plt

def render_networkx(graph, words, max_depth=2):
    G = nx.Graph()
    print("Adding nodes...")
    open_list = list(zip(words, [0] * len(words)))
    closed_list = set()
    while (len(open_list) > 0):
        node, depth = open_list.pop()
        closed_list.add(node)
        G.add_node(node)
        if (depth + 1 < max_depth):
            for neighbor in graph[node].edges:
                if not(neighbor in closed_list):
                    open_list.append((neighbor, depth + 1))


    print("Adding edges")
    for node in closed_list:
        for neighbor in graph[node].edges:
            G.add_edge(node, neighbor)
    print("Drawing...")

    pos = nx.spring_layout(G, k=0.25)
    nx.draw_networkx_edges(G, pos)
    nx.draw_networkx_labels(G, pos)
    nx.draw_networkx_nodes(G, pos)
    plt.show()


def gen_json(graph):
    edges = {}
    for key in graph:
        edges[key] = [{"word":target, "docs":[idx for idx in graph[key].edges[target]]} for target in graph[key].edges]
    return json.dumps(edges)

def render_html(docs_data, graph, rendered_name):
    docs_json = json.dumps(docs_data)
    data = gen_json(graph)
    with open("scripts/html_render/render_template.html", "r") as file:
        rendered_content = file.read().replace("<JSON_GRAPH_DATA>", data).replace("<JSON_DOCS_DATA>", docs_json)
        with open("scripts/html_render/" + rendered_name, "w") as rendered_file:
            rendered_file.write(rendered_content)
