ad_messages = set()
for line in open("/home/ethanlam/Desktop/LearnProgramming/AdAnalyzer/Bloomberg2-4-2020"):
    ad_messages.add(line)


f = open("/home/ethanlam/Desktop/LearnProgramming/AdAnalyzer/Bloomberg2-4-2020_CLEAN", "a")
f.writelines(ad_messages)