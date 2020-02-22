function fetchJsonData(url) {
    return fetch(url)
                .then(response => {
                    if (!response.ok) {
                        throw new Error(response.status);
                    }
                    return response.json()
                });
}
// Global state shared with the graph render and script that handles ui events
GLOBAL_STATE = {
    // Has the data been loaded
    ready: false,

    // Association graph data
    data: {},
    // All documents in the corpus with ith document having id i
    docs: [],
    // Holds demographic stats for ads
    ad_stats: {},

    interesting_words: [],

    // force-graph object
    Graph: {},

    // Words that the user is exploring
    root: null,
    expandedNodes: new Set()
};

// Returns a graph for force-graph to render based on the root word
// and all words expanded. Collapsed words might cause other words to leave
// the graph.
function getVisibleTree() {
    if (GLOBAL_STATE.root == null) {
        return;
    }

    var openSet = [GLOBAL_STATE.root];

    visibleNodes = new Set();
    visibleLinks = new Set();
    // Execute a DFS to find all visible nodes
    while (openSet.length > 0) {
        var current = openSet.pop();
        
        visibleNodes.add(current);
        GLOBAL_STATE.data[current].forEach(neighborData => {
            neighbor = neighborData["word"];
            if (!visibleNodes.has(neighbor)) {
                if (GLOBAL_STATE.expandedNodes.has(neighbor)) {
                    openSet.push(neighbor);
                } else {
                    visibleNodes.add(neighbor);
                }
            }
            visibleLinks.add({source: current, target: neighbor});
            visibleLinks.add({source: neighbor, target: current});
        });
    }

    // Take intersection of closed set and expanded nodes ... to prune off some expanded nodes
    var newExpandedNodes = new Set();
    GLOBAL_STATE.expandedNodes.forEach(elem => {
        if (visibleNodes.has(elem)) {
            newExpandedNodes.add(elem);
        }
    });
    GLOBAL_STATE.expandedNodes = newExpandedNodes;
    return {nodes: Array.from(visibleNodes).map(function (elem) { return {id: elem, expanded: GLOBAL_STATE.expandedNodes.has(elem)} }), links: Array.from(visibleLinks)};            
}

function renderGraph() {
    var hoveredNode = null;
    var hoveredLink = null;
    var graph = getVisibleTree();
    const elem = document.getElementById('graph');
    const Graph = ForceGraph()(elem)
        .graphData(graph)
        .nodeId('id')
        .nodeAutoColorBy('group')
        .onNodeHover(node => {
            if (node) {
                document.body.style.cursor = "pointer";
            } else {
                document.body.style.cursor = "default";
            }
            hoveredNode = node ? node : null;
        })
        .onLinkHover(link => {
            if (link) {
                document.body.style.cursor = "pointer";
            } else {
                document.body.style.cursor = "default";
            }
            hoveredLink = link ? link : null;
        })
        .linkWidth(link => (hoveredLink != null && link.source == hoveredLink.source && link.target == hoveredLink.target) ? 7 : 2)
        .nodeRelSize(5)
        .onNodeClick(node => {
            if (GLOBAL_STATE.expandedNodes.has(node.id)) {
                GLOBAL_STATE.expandedNodes.delete(node.id);
            } else {
                GLOBAL_STATE.expandedNodes.add(node.id);
            }
            const newGraph = getVisibleTree();

            Graph.graphData(newGraph);
        })
        .onLinkClick(link => {
            // Display documents on sidebar when a link is clicked
            const source = link.source.id;
            const target = link.target.id;
            GLOBAL_STATE.data[source].forEach(neighborData => {
                if (neighborData["word"] == target) {
                    const doc_ids = neighborData["docs"];
                    const list_element = $("#doc_data");
                    list_element.html("");
                    doc_ids.forEach(id => {
                        var doc_item = `<li class="ad_doc" id="doc_${id}">${GLOBAL_STATE.docs[id]}</li>`
                        list_element.append(doc_item);
                    });
                }
            });
            $("#docs").scrollTop(0);
        })
        .nodeCanvasObject((node, ctx, globalScale) => {
            const label = node.id;
            const fontSize = 16/globalScale;
            ctx.font = `${fontSize}px Sans-Serif`;
            const textWidth = ctx.measureText(label).width;
            const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding

            ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
            // Make rectangle change colors when node is hovered over
            if (hoveredNode != null && hoveredNode.id == node.id) {
                ctx.fillStyle = "#bdd4e7cc";
            }
            ctx.fillRect(node.x - bckgDimensions[0] / 2, node.y - bckgDimensions[1] / 2, ...bckgDimensions);

            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillStyle = (node.expanded) ? "#235789" : "#020100";
            ctx.fillText(label, node.x, node.y);
        });
        GLOBAL_STATE.Graph = Graph;
}

// Load graph data and document/corpus data ... ID is loaded when the HTML file is rendered
fetchJsonData(`/explore/${ID}/graph`)
.then(data => {
    fetchJsonData(`/explore/${ID}/corpus`)
    .then(corpus => {
        fetchJsonData(`/explore/${ID}/stats`)
        .then(ad_stats => {
            fetchJsonData(`/explore/${ID}/interesting_words/20`)
            .then(interesting_words => {
                GLOBAL_STATE = {
                    ready: true,
    
                    data: data,
                    docs: corpus["raw_corpus"],
                    ad_stats: ad_stats,
                    interesting_words: interesting_words,
                
                    root: null,
                    expandedNodes: new Set(),
                };
                renderGraph();
                $("#select_word_button").text("Explore word!");
                $("#select_word_button").prop('disabled', false);
            })
        })
    });            
});