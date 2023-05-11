#include <bits/stdc++.h>
using namespace std;

int hamming(const string &s1, const string &s2) {
  if (s1.size() != s2.size())
    return INT_MAX;

  int ans = 0;
  for (size_t i = 0; i < s1.size(); ++i)
    ans += s1[i] != s2[i];

  return ans;
}

void autocomplete1(const map<string, int> &words, const string &word) {
  auto it = words.lower_bound(word);
  bool found = false;
  if (it->first.substr(0, word.size()) == word) {
    cout << it->first << '\n';
    found = true;
  }

  if (!found)
    cout << "No words found\n";
}

void autocomplete2(const map<string, int> &words, const string &word) {
  string ans = "";

  for (auto it = words.lower_bound(word); it != words.end(); ++it)
    if ((it->first.size() < ans.size() || ans.empty())) {
      if (it->first.substr(0, word.size()) == word)
        ans = it->first;
      else
        break;
    }

  cout << (ans.empty() ? "No words found" : ans) << '\n';
}

void autocomplete3(const map<string, int> &words, const string &word) {
  string ans = "";
  int max_freq = 0;

  for (auto it = words.lower_bound(word); it != words.end(); ++it)
    if (it->second > max_freq) {
      if (it->first.substr(0, word.size()) == word) {
        ans = it->first;
        max_freq = it->second;
      } else
        break;
    }

  cout << (ans.empty() ? "No words found" : ans) << '\n';
}

int main() {
  string line, command, arg1, arg2;
  map<string, int> words;

  while (1) {
    getline(cin, line);
    istringstream(line) >> command >> arg1 >> arg2;

    if (command == "INSERT")
      ++words[arg1];
    else if (command == "REMOVE")
      words.erase(arg1);
    else if (command == "AUTOCORRECT") {
      int k = stoi(arg2);
      bool found = false;

      for (const auto &[word, freq] : words)
        if (hamming(word, arg1) <= k) {
          cout << word << '\n';
          found = true;
        }

      if (!found)
        cout << "No words found\n";
    } else if (command == "LOAD") {
      ifstream fin(arg1);
      string word;

      while (fin >> word)
        ++words[word];
    } else if (command == "AUTOCOMPLETE") {
      int k = stoi(arg2);
      if (k == 0) {
        autocomplete1(words, arg1);
        autocomplete2(words, arg1);
        autocomplete3(words, arg1);
      } else if (k == 1)
        autocomplete1(words, arg1);
      else if (k == 2)
        autocomplete2(words, arg1);
      else if (k == 3)
        autocomplete3(words, arg1);
    } else
      return 0;
  }
}
