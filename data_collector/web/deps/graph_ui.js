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

    // Dictionaries holding each group and list of documents sorted by impressions
    region_filter: {},
    demographic_filter: {},

    // Resource used to render choropleth charts
    us_geo_data: {},

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

function displayDocuments(doc_ids) {
    const list_element = $("#doc_data");
    list_element.html("");

    doc_ids.forEach(id => {
        var doc_item = `<li class="ad_doc" id="doc_${id}">${GLOBAL_STATE.docs[id]}</li>`
        list_element.append(doc_item);
    });

    $("#docs").scrollTop(0);
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
                    
                    displayDocuments(doc_ids);
                }
            });
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

function renderAggregateRegionChart() {
    var region_ctx = document.getElementById("region_explorer_chart").getContext("2d");

    const us = GLOBAL_STATE.us_geo_data;
    const nation = ChartGeo.topojson.feature(us, us.objects.nation).features[0];
    const states = ChartGeo.topojson.feature(us, us.objects.states).features;
    
    const region_labels = Object.keys(GLOBAL_STATE.region_filter).map(label => {
        return `${label} (Estimated Total Impressions: ${GLOBAL_STATE.region_filter[label].estimated_impressions.toFixed()})`;
    });
    const region_data = Array.from(Object.keys(GLOBAL_STATE.region_filter).map(label => {
        return {
            feature: states.find((d) => d.properties.name === label),
            value: GLOBAL_STATE.region_filter[label].estimated_impressions
        };
    }));
    var max = 1;
    Object.keys(GLOBAL_STATE.region_filter).forEach(label => {
        value = GLOBAL_STATE.region_filter[label].estimated_impressions;
        if (value > max) {
            max = value;
        }
    })

    const region_aggregate_chart = new Chart(region_ctx, {
        type: 'choropleth',
        data: {
            labels: region_labels,
            datasets: [{
                outline: nation,
                data: region_data,
                backgroundColor: (context) => {
                    if (context.dataIndex == null) {
                        return null;
                    }
                    const value = context.dataset.data[context.dataIndex];
                    const percent = value.value/max;
                    const color = new Color().rgb(255 - percent*255, 255 - percent*255, 255 - percent*255);                       
                    return color.rgbString();
                },
                borderWidth: 1,
            }],
        },
        options: {
            legend:{
                display:false
            },
            scale: {
                projection: 'albersUsa'
            },
            onClick: (evt, elems) => {
                const label = elems.map((elem) => elem.feature.properties.name)[0];
                const doc_ids = GLOBAL_STATE.region_filter[label].documents.map((o) => o.doc_id);
                displayDocuments(doc_ids);
            }
        }
    });
    // Get canvas to fill container div
    $("#region_explorer_chart").css("width", '100%');
    $("#region_explorer_chart").css("height", '100%');
    $("#region_explorer_chart").width  = $("#region_explorer_chart").offsetWidth;
    $("#region_explorer_chart").height = $("#region_explorer_chart").offsetHeight;
}

function renderAggregateDemographicChart() {
    var demographic_ctx = document.getElementById("demographic_explorer_chart").getContext("2d");
    
    const demographic_labels = Object.keys(GLOBAL_STATE.demographic_filter).sort();
    const demographic_data = Array.from(demographic_labels.map(label => {
        const val = GLOBAL_STATE.demographic_filter[label];
        return [val.total_impression_lower, val.total_impression_upper];
    }));

    const female_color = "#F50057";
    const male_color = "#2979FF";
    const unknown_color = "#CCCCCC"
    const demographic_colors = Array.from(demographic_labels.map(label => {
        const gender = label.split("/")[0];
        if (gender == "male") {
            return male_color;
        } else if (gender == "female") {
            return female_color;
        } else {
            return unknown_color;
        }
    }));

    const aggregate_demographic_chart = new Chart(demographic_ctx, {
        type: 'bar',
        data: {
            labels: demographic_labels,
            datasets: [{
                data: demographic_data,
                backgroundColor: demographic_colors,
                borderWidth: 1,
            }],
        },
        options: {
            legend:{
                display:false
            },
            scales: {
                yAxes: [{
                    ticks: {
                        beginAtZero: false
                    },
                    scaleLabel: {
                        display: true,
                        labelString: "Estimated # of Impressions"
                    }
                }],
                xAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: "Demographic"
                    }
                }]
            },
            onClick: ((e, item) => {
                const label_idx = item[0]._index;
                const label = demographic_labels[label_idx];
                const doc_ids = GLOBAL_STATE.demographic_filter[label].documents.map((o) => o.doc_id);
                displayDocuments(doc_ids);
            })
        }
    });
    // Get canvas to fill container div
    $("#demographic_explorer_chart").css("width", '100%');
    $("#demographic_explorer_chart").css("height", '100%');
    $("#demographic_explorer_chart").width  = $("#demographic_explorer_chart").offsetWidth;
    $("#demographic_explorer_chart").height = $("#demographic_explorer_chart").offsetHeight;
    
}

// Group documents into different demographics/regions and sort
function groupDocuments() {
    GLOBAL_STATE.docs.forEach((doc_text, doc_id) => {
        const doc_stats = GLOBAL_STATE.ad_stats[doc_text];
        const demographic = doc_stats["demographic_impression"];
        const region = doc_stats["region_impression"];

        // Group documents into different demographics
        Object.keys(demographic).forEach(group => {
            if (GLOBAL_STATE.demographic_filter[group] == null) {
                GLOBAL_STATE.demographic_filter[group] = {
                    total_impression_lower: 0,
                    total_impression_upper: 0,
                    documents: [],
                };
            }

            GLOBAL_STATE.demographic_filter[group].total_impression_lower += demographic[group][0];
            GLOBAL_STATE.demographic_filter[group].total_impression_upper += demographic[group][1];
            GLOBAL_STATE.demographic_filter[group].documents.push({
                doc_id: doc_id,
                estimated_impressions: (demographic[group][0] + demographic[group][1]) / 2.0
            });
        });
        // Sort documents in each demographic group
        Object.keys(GLOBAL_STATE.demographic_filter).forEach(group => {
            GLOBAL_STATE.demographic_filter[group].documents.sort((doc1_data, doc2_data) => {
                // Sort in DESCENDING ORDER ... higher values return -1 "less than"
                if (doc1_data.estimated_impressions > doc2_data.estimated_impressions) {
                    return -1;
                }
                if (doc1_data.estimated_impressions < doc2_data.estimated_impressions) {
                    return 1;
                } else {
                    return 0;
                }
            });
        });
        // Group documents into different region locations
        Object.keys(region).forEach(location => {
            if (GLOBAL_STATE.region_filter[location] == null) {
                GLOBAL_STATE.region_filter[location] = {
                    estimated_impressions: 0,
                    documents: [],
                };
            }

            const estimated_impressions = (region[location][0] + region[location][1]) / 2.0;
            GLOBAL_STATE.region_filter[location].estimated_impressions += estimated_impressions;
            GLOBAL_STATE.region_filter[location].documents.push({
                doc_id: doc_id,
                estimated_impressions: estimated_impressions
            });
        });
        // Sort documents in each region group
        Object.keys(GLOBAL_STATE.region_filter).forEach(location => {
            GLOBAL_STATE.region_filter[location].documents.sort((doc1_data, doc2_data) => {
                // Sort in DESCENDING ORDER ... higher values return -1 "less than"
                if (doc1_data.estimated_impressions > doc2_data.estimated_impressions) {
                    return -1;
                }
                if (doc1_data.estimated_impressions < doc2_data.estimated_impressions) {
                    return 1;
                } else {
                    return 0;
                }
            });
        });
    });
    renderAggregateRegionChart();
    renderAggregateDemographicChart();
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
                fetchJsonData('https://unpkg.com/us-atlas/states-10m.json')
                .then((us) => {
                    GLOBAL_STATE = {
                        ready: true,
        
                        data: data,
                        docs: corpus["raw_corpus"],
                        ad_stats: ad_stats,
                        interesting_words: interesting_words,
    
                        region_filter: {},
                        demographic_filter: {},
    
                        Graph: {},

                        us_geo_data: us,
                    
                        root: null,
                        expandedNodes: new Set(),
                    };
                    renderGraph();
                    groupDocuments();
                    $("#select_word_button").text("Explore word!");
                    $("#alternate_explore").text("Explore by Demographic/Region!");
                    $("#select_word_button").prop('disabled', false);
                    $("#alternate_explore").prop('disabled', false);
                })
            })
        })
    });            
});