
## Population


### Hierarchy & Terminology

 * Note on settlement hierarchy
    * Village
        * Aka: 
            * Village, Hamlet (Smaller, without market)
    * Town
        * Default Aka: Town, Borough ( Fortified / Castle-town)
    * City
    
 * Note on political hierarchy
    * Township
        * Aka Township, Barony
    * County
        * Aka
            * Default: County, March (border, larger), Shire (less metropolitan / cultural?)
            * North: County, Mark (border, larger)
    * Duchy
        * Aka:
            * Default:
                * Duchy, Petty Kingdom (if not member of larger kingdom), Earldom
            * Alt:
                * Principality
            * North:
                * Jarldom
    * Kingdom
        * Aka:
            * Kingdom, Grand Duchy (Smaller / composed of single duchy)
    * Empire
        Multiple regions / cultural groups. May contain kingdoms

### Operation

 * Determine settlement populations
    * Find subsistence settlement locations
        * Villages, Hamlets, similar.
    * Create townships
        * Grouping of villages which are associated with a single town.
        * Promote village to town which has best connectivity.
    * Create Urbanities
        * Grouping of towns which are associated with a City.
        * Promote town to city which has best connectivity.


#### Village & Hamlet Creation

Villages & Hamlets are subsistence settlements - self sufficient 
settlements that are the basis for all other settlements.

##### Placement

 * Scatter nodes across a region.
    * Randomized Hex graph?
    * 'Snap to' rivers and streams.
 * Estimate area population density based on food supply.
    * Will be ~100 for areas where land is highly arable w/ good climate. (Medieval France)
    * Will be ~30 where wooded with a moderately good climate (Medieval Britain)
    * Wooded land: ~90 For medieval Germany
    * 'Pure' pastureland will support ~180 people.
    * Fishing should be considered a ready source of food on par with ideal arable land.
 * Generate population w/ bias from food abundence (50 to 300 normal, 20-1000 possible)
    * Note: Smaller hamlets (Non-grain based settlements should be connected to grainfields)
 * Once town & city road connections are established, villages may snap
        to these roads if nearby.

#### Town & City Creation

 * Group subsistence villages into townships
    * May be done by selecting villages at random, and then 'growing' an influence radius
            until sufficient villages are included to supply the town.
        * If insufficient villages are in range, simply produce a 
                smaller town or larger village.
        * Find village with best connectivity 
            * (On largest river or coast & most centrally located)
        * Promote best connected village to town.
    * Estmate population based on supply from villages in influence range.
        * May be in range of 1000 - 8000 with 2500 Avg.

 * Repeat process for Cities, with towns as suppliers.
    * Population range of 8000 to 12000 with 10000 avg.
 * Most central city in a region may be promoted to Supercity, with
        population up to 100,000 (Historical examples: 
        London (25-40k), Paris (50-80k) Genoa (75-100k), Venice (100+k)
    * This will be a region (Realm) capital.


### Village, Town, & City Generation
 * Start with initial road(s).
 * Determine structures to place (businesses, etc)
 * Place initial nodes & obstacles
 * Added objects add ObstacleLines and Nodes
    * Obstacle lines must not be intersected by streets.
    * Nodes are potential points through which future streets may pass.


### Population breakdown

 * Classes:
    * 42% Serfs
    * 40% Tenant Farmers / Craftsmen, etc
    * 12% Landowners / 'freemen'
    * 5% Knights & similar lower nobility.
    * 1% Nobility
    

###
