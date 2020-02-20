function hideModal() {
    $("#veil").removeClass("darken_bg");
    current_modal.css("display", "none");
    current_modal = null;
}

function submitWord() {
    const word = $("#input_word").val().toLowerCase().trim();
    if (GLOBAL_STATE.data[word] == null) {
        // Display error
        $("#select_word_error").show();
        $("#input_word").val("")
    } else {
        $("#select_word_error").hide();
        // Set global state
        GLOBAL_STATE.root = word;
        GLOBAL_STATE.expandedNodes = new Set();
        GLOBAL_STATE.expandedNodes.add(word);
        // Reset and update graph
        $("#input_word").val("")
        GLOBAL_STATE.Graph.graphData(getVisibleTree());
        hideModal();

        // Display instructions
        const instructions = String.raw`
        <li>Words that are connected are related to each other in the context of some ad.</li>
        <li>Click on words to expand them and to explore more related words.</li>
        <li>Click on a link connecting two words to bring up the ad(s) that contain these two words in context. Ads will appear in this box.</li>
        `;
    
        $("#doc_data").html(instructions);
    }
}

var us_choropleth = null;
function renderRegionChart(region) {
    var region_ctx = document.getElementById("us_choropleth").getContext("2d");

    fetchJsonData('https://unpkg.com/us-atlas/states-10m.json').then((us) => {
        const nation = ChartGeo.topojson.feature(us, us.objects.nation).features[0];
        const states = ChartGeo.topojson.feature(us, us.objects.states).features;
        
        const region_labels = Object.keys(region).map(label => {
            const lower = region[label][0];
            const upper = region[label][1];
            return `${label} (Impressions: ${lower.toFixed()} - ${upper.toFixed()})`;
        });
        const region_data = Array.from(Object.keys(region).map(label => {
            return {
                feature: states.find((d) => d.properties.name === label),
                value: (region[label][0] + region[label][1]) / 2.0
            };
        }));
        var max = 1;
        Object.keys(region).forEach(label => {
            value = (region[label][0] + region[label][1]) / 2.0;
            if (value > max) {
                max = value;
            }
        })
        console.log(Array.from(states.map((d) => d.properties.name)));


        if (us_choropleth != null) {
            // Destroy previous chart
            us_choropleth.destroy();
        }
        us_choropleth = new Chart(region_ctx, {
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
                }
            }
        });
        // Get canvas to fill container div
        $("#us_choropleth").css("width", '100%');
        $("#us_choropleth").css("height", '100%');
        $("#us_choropleth").width  = $("#demographic_chart").offsetWidth;
        $("#us_choropleth").height = $("#demographic_chart").offsetHeight;
    });
}


var demographic_chart = null;
function renderDemographicChart(demographic) {
    var demographic_ctx = document.getElementById("demographic_chart").getContext("2d");
    
    const demographic_labels = Object.keys(demographic).sort();
    const demographic_data = Array.from(demographic_labels.map(label => {
        return demographic[label];
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

    if (demographic_chart != null) {
        // Destroy previous chart
        demographic_chart.destroy();
    }
    demographic_chart = new Chart(demographic_ctx, {
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
            }
        }
    });
    // Get canvas to fill container div
    $("#demographic_chart").css("width", '100%');
    $("#demographic_chart").css("height", '100%');
    $("#demographic_chart").width  = $("#demographic_chart").offsetWidth;
    $("#demographic_chart").height = $("#demographic_chart").offsetHeight;
    
}

current_modal = null;
$(document).ready(function() {
    $("#select_word_button").click(function() {
        // Don't load modal if all data hasn't been collected
        if (GLOBAL_STATE.ready) {
            // Get modal ready for displaying
            $("#veil").addClass("darken_bg");
            current_modal = $("#select_word")
            $("#select_word").css("display", "block");
            $("#input_word").focus();

            for (var i = 0; i < GLOBAL_STATE.interesting_words.length; i++) {
                const element = GLOBAL_STATE.interesting_words[i].toLowerCase().trim();
                if (GLOBAL_STATE.data[element] != null) {
                    $("#suggested_words").append('<li class="list-group-item">'+ element +'</li>');
                }
            }
        }
    });
    // Create click event listener as ads are not loaded until later
    $(document).on("click", ".ad_doc", function() {
        const doc_id = parseInt($(this).attr('id').split("_")[1]);
        // Convert doc_id to doc text and use that to lookup the stats
        const doc_stats = GLOBAL_STATE.ad_stats[GLOBAL_STATE.docs[doc_id]];
        const demographic = doc_stats["demographic_impression"];
        const region = doc_stats["region_impression"];
        console.log(doc_stats);
        // Popup modal
        if (current_modal != null) {
            // Dismiss whatever modal is currently opened
            hideModal();
        }
        $("#veil").addClass("darken_bg");
        current_modal = $("#stats_modal")
        $("#stats_modal").css("display", "block");

        renderDemographicChart(demographic);
        renderRegionChart(region);
    });

    $("#veil").click(hideModal);
    $("#submit_word").click(submitWord);
    $("#input_word").keypress(function(e) {
        if(e.which == 13) {
            submitWord();
        }
    });
});