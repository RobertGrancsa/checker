# Tema 3

## Task 1 - Smart Keyboard (50p)

Ne propunem sa implementam un sistem de corectare/completare a cuvintelor pentru un utilizator in baza istoricului lui (cuvinte folosite in trecut). Scopul nostru este sa folosim eficient structurile de date pentru a realiza cat mai rapid operatiile.

## Operatii

Se garanteaza corectitudinea formatului operatiilor.

### Insert

Retinem cuvantul sau incrementam frenventa de aparitie in cadrul structurii de date.

`INSERT <cuvant>`

### Load

Citim un fisier ascii (acesta va contine litere mici ale alfabetului englez si whitespace), si inseram toate cuvintele din el in structura de date. Acest fisier modeleaza istoricul utilizatorului.

Se garanteaza ca fisierul exista si este formatat corect.

`LOAD <filename>`

In urma operatiei, se va afisa mesajul "File <filename> succesfully loaded\n"

### Remove

Eliminam cuvantul (si eliberam memoria folosita de acesta) din structura de date (totusi acesta poate fi readaugat in viitor).

`REMOVE <cuvant>`

#### Erori

- Atentie, cuvantul poate sa nu existe, caz in care nu se va intampla nimic

### Autocorrect

Se cer toate cuvintele care difera de cuvantul dat, in maxim k caractere. Vom lua in calcul doar schimbari de litere, nu si inserari sau stergeri.

Se vor afisa toate cuvintele in ordine lexicografica.

Exemplu

```
> INSERT mask
> INSERT mass
> INSERT man
> INSERT bass
> INSERT marks
> AUTOCORRECT mars 1
< mass
> AUTOCORRECT mars 2
< bass
< mask
< mass
```

`AUTOCORRECT <cuvant> <k>`

#### Erori

- Daca nu exista cuvantul, se va afisa: "No words found\n"

### Autocomplete

Pentru acest task, aveti de afisat 3 cuvinte:

1. Cel mai mic lexicografic cuvant cu prefixul dat
1. Cel mai scurt cuvant cu prefixul dat
1. Cel mai frecvent folosit cuvant cu prefixul dat (in caz de egalitate, cel mai mic lexicografic)

`AUTOCOMPLETE <prefix> <nr-criteriu>`

Afisare:

```
> INSERT mama
> INSERT matei
> INSERT mar
> INSERT mare
> INSERT matei
> AUTOCOMPLETE ma 0
< mama
< mar
< matei
> AUTOCOMPLETE ma 1
< mama
> AUTOCOMPLETE ma 2
< mar
> AUTOCOMPLETE ma 3
< matei

```

#### Atentie - Cuvintele se pot repeta

#### Erori

- Daca nu exista cuvantul, se va afisa: "No words found\n"
- In cazul apelului cu parametrul 0, se poate afisa "No words found\n" de mai multe ori

### Exit

Eliberam memoria si terminam programul.

`EXIT`

## Task 2 - kNN (50p)

Tema cere implementarea structurii de date k-d trees, o generalizare a ABC pentru date multidimensionale. Ideea din spate este extrem de simpla, si anume:

- pe nivelul 1 vom imparti dupa prima dimensiune:
    - subarborele stang va contine punctele mai mici decat radacina
    - subarborele drept va contine punctele mai mari sau egale decat radacina (fara radacina)
- pe nivelul 2 vom imparti dupa a doua dimensiune:
    - subarborele stang va contine punctele mai mici decat radacina
    - subarborele drept va contine punctele mai mari sau egale decat radacina (fara radacina)
- ...
- pe nivelul d vom imparti dupa a (d % k + 1) dimensiune:
    - subarborele stang va contine punctele mai mici decat radacina
    - subarborele drept va contine punctele mai mari sau egale decat radacina (fara radacina)

![](https://opendsa-server.cs.vt.edu/ODSA/Books/CS3/html/_images/KDtree.png)

![alt-text](https://upload.wikimedia.org/wikipedia/commons/3/36/Kdtree_animation.gif)

## Operatii

### Load

Programul va incepe mereu cu operatia de load. Nu se vor mai adauga puncte dupa aceasta prima incarcare a datelor. Coordonatele punctelor sunt numere intregi.

`LOAD <filename>`

Fisierul o sa aibe datele in forma:

```
n k
a_11 a_12 a_13 ... a_1k
a_21 a_22 a_23 ... a_2k
...
a_n1 a_n2 a_n3 ... a_nk


Unde pe fiecare linie avem cate un vector cu k intrari (pe acesta il vom numi in continuare punct)
```

Precizare:

-10000 <= a_ij <= 10000

Se vor incarca punctele din fisier in structura de date.

### NN - Nearest Neighbour

Se cere gasirea si afisarea celui mai apropiat punct de punctul dat. Vom folosi distanta euclidiana intre puncte:

#### TODO - Insert formula here

Formal, dorim punctul ce minimizeaza distanta intre el insusi si b.

`NN <b_1> <b_2> ... <b_k>`

Afisare:
```
c_1 c_2 c_3 ... c_k
```

### Range search

Se cere gasirea si afisarea punctelor ale caror coordonate se incadreaza in intervalele date.

`RS <start_1> <end_1> <start_2> <end_2> ... <start_k> <end_k>`

Afisare:

```
m:
c_11 c_12 c_13 ... c_1k
c_21 c_22 c_23 ... c_2k
...
c_m1 c_m2 c_m3 ... c_mk

Unde
    m = numarul de puncte gasite
    pe urmatoarele m linii avem punctele gasite
```

### Exit

Eliberam memoria si terminam programul.

`EXIT`

## Referinte

[k-d trees pe wikipedia](https://en.wikipedia.org/wiki/K-d_tree)
