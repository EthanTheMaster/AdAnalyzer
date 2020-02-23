#!/bin/bash 
MODEL_NAME="<CAMPAIGN NAME>"
IP_ADDRESS="127.0.0.1:8080"
################# Ad collector parameters #################
ACCESS_TOKEN="<SECRET TOKEN>"
PAGE_IDS="id1,id2,id3,..."

YEAR_START=2020
MONTH_START=1
DAY_START=1

YEAR_END=2020
MONTH_END=12
DAY_END=31

RETRIES=5
BATCH_SIZE=500
AD_STATUS="ALL"
################# NLP Model Parameters #################
NUM_TOPICS=20
DOC2VEC_EPOCHS=500
DOC2VEC_WORKERS=8

# Run ad collector
cd data_collector/
SAVE_PATH="./web/data/${MODEL_NAME}"
cargo run --release collect --save_path=$SAVE_PATH --access_token=$ACCESS_TOKEN --ad_status=$AD_STATUS --page_ids=$PAGE_IDS --batch_size=$BATCH_SIZE --retries=$RETRIES --year_start=$YEAR_START --month_start=$MONTH_START --day_start=$DAY_START --year_end=$YEAR_END --month_end=$MONTH_END --day_end=$DAY_END

# Run analysis scripts on the ad data
NLP_MODEL_PATH="${SAVE_PATH}/models" 
python3 ../scripts/preprocess.py "${SAVE_PATH}/ad_data.json" $NLP_MODEL_PATH --num_topics=$NUM_TOPICS --doc2vec_epochs=$DOC2VEC_EPOCHS --doc2vec_workers=$DOC2VEC_WORKERS
python3 ../scripts/associate_words.py "${NLP_MODEL_PATH}/corpus_data.json" $SAVE_PATH

# Launch web server
echo "Explore data at: ${IP_ADDRESS}/explore/${MODEL_NAME}"
cargo run --release launch $IP_ADDRESS