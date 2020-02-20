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
                    console.log(element);
                    $("#suggested_words").append('<li class="list-group-item">'+ element +'</li>');
                }
            }
        }
    });
    $("#veil").click(hideModal);
    $("#submit_word").click(submitWord);
    $("#input_word").keypress(function(e) {
        if(e.which == 13) {
            submitWord();
        }
    });
});