import networkx as nx
import matplotlib.pyplot as plt

def render(graph, words, max_depth=2):
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
